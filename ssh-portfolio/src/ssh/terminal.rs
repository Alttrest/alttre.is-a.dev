//! Ratatui terminal that renders into an SSH channel instead of a local TTY.
//!
//! `TerminalHandle` buffers writes and, on `flush()`, hands the bytes to a
//! bounded channel drained by an async writer task which forwards them with
//! `russh::server::Handle::data`. We NEVER touch the server's own terminal
//! (no `enable_raw_mode`, no `event::read`).

use std::io::{self, Write};

use ratatui::layout::Rect;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use russh::ChannelId;
use russh::server::Handle;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::ui::{self, App};

/// Bounded queue of frames between the renderer and the SSH writer task.
pub const WRITE_QUEUE_CAP: usize = 64;
/// If a slow client lets pending output exceed this, we drop the session
/// rather than grow memory without bound.
pub const MAX_PENDING_BYTES: usize = 2 * 1024 * 1024;

pub const MIN_COLS: u16 = 20;
pub const MAX_COLS: u16 = 500;
pub const MIN_ROWS: u16 = 10;
pub const MAX_ROWS: u16 = 200;

/// Enter alt screen, hide cursor, enable X10 + SGR mouse reporting.
pub const ENTER_SEQ: &str = "\x1b[?1049h\x1b[?25l\x1b[?1000h\x1b[?1006h";
/// Undo `ENTER_SEQ` and reset attributes.
pub const RESTORE_SEQ: &str = "\x1b[?25h\x1b[?1000l\x1b[?1006l\x1b[?1049l\x1b[0m";

/// Clamp an arbitrary client-reported size into a sane range.
pub fn clamp_size(cols: u32, rows: u32) -> (u16, u16) {
    (
        cols.clamp(MIN_COLS as u32, MAX_COLS as u32) as u16,
        rows.clamp(MIN_ROWS as u32, MAX_ROWS as u32) as u16,
    )
}

/// `io::Write` sink that ships flushed bytes to a writer task.
pub struct TerminalHandle {
    sink: Vec<u8>,
    tx: mpsc::Sender<Vec<u8>>,
    /// Set once the channel closed: later writes are discarded silently so
    /// ratatui's `Drop` (show cursor) cannot spam errors.
    dead: bool,
}

impl TerminalHandle {
    /// Build a handle around an existing sender (used directly by tests).
    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            sink: Vec::new(),
            tx,
            dead: false,
        }
    }

    /// Create a handle plus the async task that forwards bytes to the SSH
    /// channel. Await the returned `JoinHandle` (after dropping the handle)
    /// to be sure everything was written.
    pub fn spawn(handle: Handle, channel: ChannelId) -> (Self, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(WRITE_QUEUE_CAP);
        let task = tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                if handle.data(channel, bytes).await.is_err() {
                    break;
                }
            }
        });
        (Self::new(tx), task)
    }
}

impl Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.dead {
            self.sink.clear();
            return Ok(());
        }
        if self.sink.is_empty() {
            return Ok(());
        }
        // Never take the bytes unless the queue accepted them: a diff-based
        // renderer must not lose a frame, so on `Full` we keep the data and
        // retry (coalesced) on the next flush.
        match self.tx.try_send(std::mem::take(&mut self.sink)) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(bytes)) => {
                self.sink = bytes;
                if self.sink.len() > MAX_PENDING_BYTES {
                    Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "client too slow: output backlog exceeded",
                    ))
                } else {
                    Ok(())
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.dead = true;
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "ssh channel closed",
                ))
            }
        }
    }
}

/// Per-session Ratatui terminal writing to an SSH channel.
pub struct SshTerminal {
    terminal: Terminal<CrosstermBackend<TerminalHandle>>,
    width: u16,
    height: u16,
}

impl SshTerminal {
    pub fn new(handle: TerminalHandle, cols: u32, rows: u32) -> io::Result<Self> {
        let (width, height) = clamp_size(cols, rows);
        let terminal = Terminal::with_options(
            CrosstermBackend::new(handle),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, width, height)),
            },
        )?;
        Ok(Self {
            terminal,
            width,
            height,
        })
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn write_raw(&mut self, s: &str) -> io::Result<()> {
        let backend = self.terminal.backend_mut();
        backend.write_all(s.as_bytes())?;
        backend.flush()
    }

    /// Alt screen + hidden cursor + SGR mouse.
    pub fn enter(&mut self) -> io::Result<()> {
        self.write_raw(ENTER_SEQ)?;
        self.terminal.clear()
    }

    /// Leave alt screen, show cursor, disable mouse, reset attributes.
    pub fn restore(&mut self) -> io::Result<()> {
        self.write_raw(RESTORE_SEQ)
    }

    /// Resize safely (clamped). Returns the effective size.
    pub fn resize(&mut self, cols: u32, rows: u32) -> io::Result<(u16, u16)> {
        let (w, h) = clamp_size(cols, rows);
        self.width = w;
        self.height = h;
        self.terminal.resize(Rect::new(0, 0, w, h))?;
        Ok((w, h))
    }

    /// Render the app into the client's terminal.
    pub fn draw(&mut self, app: &App) -> io::Result<()> {
        self.terminal.draw(|frame| ui::render(frame, app))?;
        // `Terminal::draw` may leave trailing cursor sequences buffered.
        self.terminal.backend_mut().flush()
    }

    /// Write arbitrary bytes straight through (goodbye messages etc.).
    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write_raw(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(rx: &mut mpsc::Receiver<Vec<u8>>) -> Vec<u8> {
        let mut out = Vec::new();
        while let Ok(b) = rx.try_recv() {
            out.extend(b);
        }
        out
    }

    #[test]
    fn clamp_bounds() {
        assert_eq!(clamp_size(0, 0), (20, 10));
        assert_eq!(clamp_size(80, 24), (80, 24));
        assert_eq!(clamp_size(10_000, 10_000), (500, 200));
        assert_eq!(clamp_size(u32::MAX, u32::MAX), (500, 200));
    }

    #[test]
    fn flush_sends_buffer_and_empties_sink() {
        let (tx, mut rx) = mpsc::channel(4);
        let mut h = TerminalHandle::new(tx);
        h.write_all(b"hello ").unwrap();
        h.write_all(b"world").unwrap();
        assert!(rx.try_recv().is_err(), "nothing sent before flush");
        h.flush().unwrap();
        assert_eq!(rx.try_recv().unwrap(), b"hello world");
        h.flush().unwrap(); // empty flush sends nothing
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn full_queue_keeps_data_and_coalesces() {
        let (tx, mut rx) = mpsc::channel(1);
        let mut h = TerminalHandle::new(tx);
        h.write_all(b"one").unwrap();
        h.flush().unwrap();
        h.write_all(b"two").unwrap();
        h.flush().unwrap(); // queue full -> kept, no error
        h.write_all(b"three").unwrap();
        assert_eq!(rx.try_recv().unwrap(), b"one");
        h.flush().unwrap();
        assert_eq!(rx.try_recv().unwrap(), b"twothree");
    }

    #[test]
    fn slow_client_backlog_errors_instead_of_oom() {
        let (tx, _rx) = mpsc::channel(1);
        let mut h = TerminalHandle::new(tx);
        h.write_all(b"x").unwrap();
        h.flush().unwrap();
        h.write_all(&vec![0u8; MAX_PENDING_BYTES + 1]).unwrap();
        assert!(h.flush().is_err());
    }

    #[test]
    fn closed_channel_is_broken_pipe() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        let mut h = TerminalHandle::new(tx);
        h.write_all(b"x").unwrap();
        assert_eq!(h.flush().unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    }

    #[test]
    fn enter_draw_resize_restore() {
        // Large queue: this test never drains while resizing.
        let (tx, mut rx) = mpsc::channel(8192);
        let mut t = SshTerminal::new(TerminalHandle::new(tx), 80, 24).unwrap();
        assert_eq!(t.size(), (80, 24));
        t.enter().unwrap();
        let out = String::from_utf8_lossy(&drain(&mut rx)).to_string();
        assert!(out.starts_with(ENTER_SEQ), "got {out:?}");

        let app = App::new();
        t.draw(&app).unwrap();
        assert!(!drain(&mut rx).is_empty(), "draw must emit bytes");

        assert_eq!(t.resize(5, 1).unwrap(), (20, 10));
        t.draw(&app).unwrap();
        assert_eq!(t.resize(9999, 9999).unwrap(), (500, 200));
        t.draw(&app).unwrap();
        assert_eq!(t.resize(120, 40).unwrap(), (120, 40));
        t.draw(&app).unwrap();
        drain(&mut rx);

        t.restore().unwrap();
        let out = String::from_utf8_lossy(&drain(&mut rx)).to_string();
        assert_eq!(out, RESTORE_SEQ);
    }
}
