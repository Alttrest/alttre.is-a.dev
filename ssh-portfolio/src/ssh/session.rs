//! Per-connection `russh` handler + per-session actor.
//!
//! Each connected user gets an isolated `App` + `SshTerminal` owned by a
//! dedicated tokio task (the *actor*). The SSH handler only forwards
//! `SessionCommand`s over an mpsc channel, so input, resize and the ~100ms
//! tick can never race and there is no lock anywhere.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use russh::server::{Auth, Handle, Handler, Msg, Session, VerifiedPublicKey};
use russh::{Channel, ChannelId, Pty};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{Instant, MissedTickBehavior, interval};

use super::input::InputParser;
use super::terminal::{SshTerminal, TerminalHandle, clamp_size};
use crate::events::{AppAction, InputEvent, KeyCode};
use crate::ui::App;

/// UI animation / uptime tick.
pub const TICK_INTERVAL: Duration = Duration::from_millis(100);
/// Commands queued between the SSH handler and a session actor.
const COMMAND_QUEUE: usize = 256;
/// Max commands coalesced into a single redraw.
const MAX_BATCH: usize = 64;

/// Messages from the SSH handler to the session actor.
#[derive(Debug)]
pub enum SessionCommand {
    /// Raw bytes typed by the client.
    Input(Vec<u8>),
    /// New terminal size (unclamped; the terminal clamps).
    Resize(u32, u32),
    /// Tear down immediately (client went away).
    Shutdown,
}

/// How a session actor ended.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// User quit or idle timeout: say goodbye and close the channel.
    Quit,
    /// Client / connection is gone: nothing more to say.
    Gone,
}

/// Crisp ANSI business card shown for non-interactive use.
pub fn business_card(pty: bool) -> String {
    const RED: &str = "\x1b[38;2;218;41;28m";
    const CREAM: &str = "\x1b[38;2;250;246;238m";
    const MUTED: &str = "\x1b[38;2;142;130;116m";
    const SAGE: &str = "\x1b[38;2;94;112;96m";
    const B: &str = "\x1b[1m";
    const R: &str = "\x1b[0m";
    let card = format!(
        "\n\
         {RED}{B}  /\\   {CREAM}Ali Turan{R}  {MUTED}@Alttre{R}\n\
         {RED}{B} /--\\  {R}{SAGE}Information Technology student & software developer{R}\n\
         \n\
         {MUTED}  web     {R}{CREAM}https://alttre.is-a.dev{R}\n\
         {MUTED}  github  {R}{CREAM}https://github.com/Alttrest{R}\n\
         {MUTED}  email   {R}{CREAM}ali-turan-@outlook.com{R}\n\
         \n\
         {MUTED}  Want the full interactive portfolio? Run:{R}\n\
         {RED}{B}    ssh -t <this-host>{R}  {MUTED}(in a real terminal){R}\n\
         \n"
    );
    if pty { card.replace('\n', "\r\n") } else { card }
}

/// One per SSH connection.
pub struct ClientSession {
    peer: Option<SocketAddr>,
    sessions: Arc<Semaphore>,
    idle_timeout: Duration,
    /// Channel that carries the (single) session.
    channel: Option<ChannelId>,
    /// PTY size requested by the client, if any.
    pty: Option<(u32, u32)>,
    /// Command channel into the actor once the shell started.
    tx: Option<Sender<SessionCommand>>,
    task: Option<JoinHandle<()>>,
}

impl ClientSession {
    pub fn new(
        peer: Option<SocketAddr>,
        sessions: Arc<Semaphore>,
        idle_timeout: Duration,
    ) -> Self {
        Self {
            peer,
            sessions,
            idle_timeout,
            channel: None,
            pty: None,
            tx: None,
            task: None,
        }
    }

    /// Write a short plain message, report `status`, and close the channel.
    fn finish_plain(
        session: &mut Session,
        channel: ChannelId,
        message: &str,
        status: u32,
    ) -> Result<()> {
        session.data(channel, message.as_bytes().to_vec())?;
        session.exit_status_request(channel, status)?;
        session.eof(channel)?;
        session.close(channel)?;
        Ok(())
    }
}

impl Drop for ClientSession {
    fn drop(&mut self) {
        // Dropping `tx` makes the actor exit on its own; nudge it anyway.
        if let Some(tx) = self.tx.take() {
            let _ = tx.try_send(SessionCommand::Shutdown);
        }
        tracing::debug!(peer = ?self.peer, "connection handler dropped");
    }
}

impl Handler for ClientSession {
    type Error = anyhow::Error;

    // The portfolio is public: every auth method succeeds.
    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _key: VerifiedPublicKey<'_>,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _user: &str, _password: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: russh::server::ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel.is_some() {
            // One session channel per connection; dropping `reply` rejects.
            return Ok(());
        }
        self.channel = Some(channel.id());
        reply.accept().await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.pty = Some((col_width, row_height));
        session.channel_success(channel)?;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;

        let Some((cols, rows)) = self.pty else {
            // `ssh -T host`: no terminal, so no TUI. Show the card instead.
            return Self::finish_plain(session, channel, &business_card(false), 0);
        };
        if self.tx.is_some() {
            return Ok(());
        }
        let permit = match self.sessions.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!(peer = ?self.peer, "session limit reached, rejecting");
                let msg = "\r\n  alttre.is-a.dev is busy right now (too many visitors).\r\n  \
                           Please try again in a moment.\r\n\r\n";
                return Self::finish_plain(session, channel, msg, 1);
            }
        };

        let (tx, rx) = mpsc::channel(COMMAND_QUEUE);
        self.tx = Some(tx);
        let handle = session.handle();
        let idle = self.idle_timeout;
        let peer = self.peer;
        self.task = Some(tokio::spawn(async move {
            tracing::info!(?peer, cols, rows, "session started");
            run_session(rx, handle, channel, cols, rows, idle, permit).await;
            tracing::info!(?peer, "session ended");
        }));
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        tracing::info!(peer = ?self.peer, cmd = %String::from_utf8_lossy(data), "exec request");
        // Any command (`whoami`, `--card`, ...) gets the business card.
        Self::finish_plain(session, channel, &business_card(self.pty.is_some()), 0)
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.pty = Some((col_width, row_height));
        if let Some(tx) = &self.tx {
            let _ = tx.send(SessionCommand::Resize(col_width, row_height)).await;
        }
        Ok(())
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(tx) = &self.tx {
            let _ = tx.send(SessionCommand::Input(data.to_vec())).await;
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(SessionCommand::Shutdown).await;
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(SessionCommand::Shutdown).await;
        }
        self.channel = None;
        Ok(())
    }
}

/// The per-session actor: owns `App` + `SshTerminal`, multiplexes commands
/// and the tick, redraws after each batch, and tears the channel down.
async fn run_session(
    mut rx: Receiver<SessionCommand>,
    handle: Handle,
    channel: ChannelId,
    cols: u32,
    rows: u32,
    idle_timeout: Duration,
    _permit: OwnedSemaphorePermit,
) {
    let (term_handle, writer) = TerminalHandle::spawn(handle.clone(), channel);
    let mut term = match SshTerminal::new(term_handle, cols, rows) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("terminal init failed: {e}");
            let _ = handle.close(channel).await;
            return;
        }
    };

    let outcome = drive(&mut term, &mut rx, cols, rows, idle_timeout).await;

    if outcome == Outcome::Quit {
        let _ = term.restore();
        let _ = term.write_str("Thanks for visiting alttre.is-a.dev \u{2014} bye!\r\n");
    }
    // Dropping the terminal closes the writer queue; wait until every byte
    // was handed to russh before sending exit-status / eof / close.
    drop(term);
    let _ = tokio::time::timeout(Duration::from_secs(3), writer).await;

    let _ = handle.exit_status_request(channel, 0).await;
    let _ = handle.eof(channel).await;
    let _ = handle.close(channel).await;
}

async fn drive(
    term: &mut SshTerminal,
    rx: &mut Receiver<SessionCommand>,
    cols: u32,
    rows: u32,
    idle_timeout: Duration,
) -> Outcome {
    if term.enter().is_err() {
        return Outcome::Gone;
    }
    let mut app = App::new();
    let (w, h) = clamp_size(cols, rows);
    // Let the app know the real viewport before the first frame.
    if app.handle_event(InputEvent::Resize(w, h)) == AppAction::Quit {
        return Outcome::Quit;
    }
    if term.draw(&app).is_err() {
        return Outcome::Gone;
    }

    let mut parser = InputParser::new();
    let mut tick = interval(TICK_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_activity = Instant::now();

    loop {
        let mut action = AppAction::Continue;
        let mut cmds = Vec::new();

        tokio::select! {
            cmd = rx.recv() => match cmd {
                None | Some(SessionCommand::Shutdown) => return Outcome::Gone,
                Some(c) => {
                    cmds.push(c);
                    // Coalesce whatever else is already queued into one redraw.
                    while cmds.len() < MAX_BATCH {
                        match rx.try_recv() {
                            Ok(c) => cmds.push(c),
                            Err(_) => break,
                        }
                    }
                }
            },
            _ = tick.tick() => {
                if last_activity.elapsed() >= idle_timeout {
                    return Outcome::Quit;
                }
                action = app.handle_event(InputEvent::Tick);
            }
        }

        for cmd in cmds {
            if action == AppAction::Quit {
                break;
            }
            match cmd {
                SessionCommand::Shutdown => return Outcome::Gone,
                SessionCommand::Input(bytes) => {
                    last_activity = Instant::now();
                    for ev in parser.feed(&bytes) {
                        // Safety net: Ctrl+C / Ctrl+D always end the session, even if
                        // the UI layer is busy in a modal.
                        let hard_quit = matches!(
                            ev,
                            InputEvent::Key(k) if k.modifiers.ctrl
                                && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('d'))
                        );
                        if hard_quit || app.handle_event(ev) == AppAction::Quit {
                            action = AppAction::Quit;
                            break;
                        }
                    }
                }
                SessionCommand::Resize(c, r) => {
                    last_activity = Instant::now();
                    match term.resize(c, r) {
                        Ok((w, h)) => {
                            if app.handle_event(InputEvent::Resize(w, h)) == AppAction::Quit {
                                action = AppAction::Quit;
                            }
                        }
                        Err(_) => return Outcome::Gone,
                    }
                }
            }
        }

        if action == AppAction::Quit {
            return Outcome::Quit;
        }
        if term.draw(&app).is_err() {
            return Outcome::Gone;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_mentions_identity_and_hint() {
        let c = business_card(false);
        for needle in [
            "Ali Turan",
            "@Alttre",
            "alttre.is-a.dev",
            "github.com/Alttrest",
            "ali-turan-@outlook.com",
            "ssh -t",
        ] {
            assert!(c.contains(needle), "missing {needle}");
        }
        assert!(!c.contains('\r'));
        assert!(c.contains("\x1b["));
    }

    #[test]
    fn card_pty_uses_crlf() {
        let c = business_card(true);
        assert!(!c.contains("\n") || c.replace("\r\n", "").find('\n').is_none());
        assert!(c.contains("\r\n"));
    }

    #[test]
    fn full_semaphore_refuses() {
        let sem = Arc::new(Semaphore::new(1));
        let _p = sem.clone().try_acquire_owned().unwrap();
        assert!(sem.clone().try_acquire_owned().is_err());
    }
}
