//! Pure VT100 / ANSI / UTF-8 + SGR mouse parser: raw SSH bytes -> `InputEvent`s.
//!
//! No I/O, no `russh`, no `crossterm` — fully unit-testable.
//! [`InputParser`] keeps a tiny buffer so escape sequences / UTF-8 characters
//! split across two SSH `data` packets are still decoded correctly. A *lone*
//! `ESC` byte at the end of a packet is reported immediately as `Esc`.

use crate::events::{InputEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseKind};

/// Max bytes we are willing to hold back waiting for a sequence to complete.
const MAX_PENDING: usize = 64;

/// Stateful parser (handles packets that split a sequence in two).
#[derive(Debug, Default)]
pub struct InputParser {
    pending: Vec<u8>,
}

impl InputParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed raw bytes, get zero or more events.
    pub fn feed(&mut self, data: &[u8]) -> Vec<InputEvent> {
        let mut buf = std::mem::take(&mut self.pending);
        buf.extend_from_slice(data);
        let mut events = Vec::new();
        let mut i = 0;
        while i < buf.len() {
            match parse_one(&buf[i..], &mut events) {
                Step::Consumed(n) => i += n.max(1),
                Step::Incomplete => {
                    if buf.len() - i <= MAX_PENDING {
                        self.pending = buf[i..].to_vec();
                    }
                    break;
                }
            }
        }
        events
    }
}

/// Stateless convenience wrapper: parse one complete chunk of bytes.
pub fn parse(data: &[u8]) -> Vec<InputEvent> {
    InputParser::new().feed(data)
}

enum Step {
    Consumed(usize),
    Incomplete,
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, modifiers))
}

fn plain(code: KeyCode) -> InputEvent {
    InputEvent::Key(KeyEvent::plain(code))
}

fn parse_one(b: &[u8], out: &mut Vec<InputEvent>) -> Step {
    let first = b[0];
    match first {
        0x1b => parse_escape(b, out),
        b'\r' | b'\n' => {
            out.push(plain(KeyCode::Enter));
            Step::Consumed(1)
        }
        b'\t' => {
            out.push(plain(KeyCode::Tab));
            Step::Consumed(1)
        }
        0x7f | 0x08 => {
            out.push(plain(KeyCode::Backspace));
            Step::Consumed(1)
        }
        0x00 => {
            out.push(key(KeyCode::Char(' '), KeyModifiers::CTRL));
            Step::Consumed(1)
        }
        0x01..=0x1a => {
            out.push(InputEvent::Key(KeyEvent::ctrl((b'a' + first - 1) as char)));
            Step::Consumed(1)
        }
        0x1c..=0x1f => Step::Consumed(1), // unsupported control bytes
        0x20..=0x7e => {
            out.push(plain(KeyCode::Char(first as char)));
            Step::Consumed(1)
        }
        _ => parse_utf8(b, out),
    }
}

fn parse_utf8(b: &[u8], out: &mut Vec<InputEvent>) -> Step {
    let need = match b[0] {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Step::Consumed(1), // stray continuation / invalid lead
    };
    let avail = b.len().min(need);
    // Validate continuation bytes seen so far.
    for (idx, &c) in b[1..avail].iter().enumerate() {
        if c & 0xc0 != 0x80 {
            return Step::Consumed(idx + 1); // drop invalid prefix
        }
    }
    if b.len() < need {
        return Step::Incomplete;
    }
    if let Ok(s) = std::str::from_utf8(&b[..need])
        && let Some(c) = s.chars().next()
    {
        out.push(plain(KeyCode::Char(c)));
    }
    Step::Consumed(need)
}

fn parse_escape(b: &[u8], out: &mut Vec<InputEvent>) -> Step {
    if b.len() == 1 {
        out.push(plain(KeyCode::Esc));
        return Step::Consumed(1);
    }
    match b[1] {
        b'[' => parse_csi(b, out),
        b'O' => {
            if b.len() < 3 {
                return Step::Incomplete;
            }
            let code = match b[2] {
                b'A' => Some(KeyCode::Up),
                b'B' => Some(KeyCode::Down),
                b'C' => Some(KeyCode::Right),
                b'D' => Some(KeyCode::Left),
                b'H' => Some(KeyCode::Home),
                b'F' => Some(KeyCode::End),
                b'P' => Some(KeyCode::F(1)),
                b'Q' => Some(KeyCode::F(2)),
                b'R' => Some(KeyCode::F(3)),
                b'S' => Some(KeyCode::F(4)),
                _ => None,
            };
            if let Some(code) = code {
                out.push(plain(code));
            }
            Step::Consumed(3)
        }
        0x1b => {
            // ESC ESC: first one is a standalone Esc.
            out.push(plain(KeyCode::Esc));
            Step::Consumed(1)
        }
        _ => {
            // Alt+<key>: ESC followed by a normal key.
            let mut inner = Vec::new();
            match parse_one(&b[1..], &mut inner) {
                Step::Incomplete => Step::Incomplete,
                Step::Consumed(n) => {
                    for ev in inner {
                        match ev {
                            InputEvent::Key(mut k) => {
                                k.modifiers.alt = true;
                                out.push(InputEvent::Key(k));
                            }
                            other => out.push(other),
                        }
                    }
                    Step::Consumed(1 + n)
                }
            }
        }
    }
}

fn parse_csi(b: &[u8], out: &mut Vec<InputEvent>) -> Step {
    // b = ESC [ params... final
    let mut end = 2;
    while end < b.len() {
        let c = b[end];
        if (0x40..=0x7e).contains(&c) {
            break;
        }
        if !(0x20..=0x3f).contains(&c) {
            // Malformed: abort sequence, drop what we saw so far.
            return Step::Consumed(end);
        }
        end += 1;
    }
    if end >= b.len() {
        return Step::Incomplete;
    }
    let final_byte = b[end];
    let params = &b[2..end];
    let consumed = end + 1;

    if params.first() == Some(&b'<') && (final_byte == b'M' || final_byte == b'm') {
        if let Some(ev) = parse_sgr_mouse(&params[1..], final_byte == b'M') {
            out.push(InputEvent::Mouse(ev));
        }
        return Step::Consumed(consumed);
    }

    let nums: Vec<u32> = std::str::from_utf8(params)
        .unwrap_or("")
        .split(';')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();
    let modifiers = nums
        .get(1)
        .map(|&m| decode_modifiers(m))
        .unwrap_or(KeyModifiers::NONE);

    let code = match final_byte {
        b'A' => Some(KeyCode::Up),
        b'B' => Some(KeyCode::Down),
        b'C' => Some(KeyCode::Right),
        b'D' => Some(KeyCode::Left),
        b'H' => Some(KeyCode::Home),
        b'F' => Some(KeyCode::End),
        b'Z' => {
            out.push(key(KeyCode::BackTab, KeyModifiers::SHIFT));
            return Step::Consumed(consumed);
        }
        b'P' => Some(KeyCode::F(1)),
        b'Q' => Some(KeyCode::F(2)),
        b'R' if params.is_empty() || nums.len() > 1 => Some(KeyCode::F(3)),
        b'S' => Some(KeyCode::F(4)),
        b'~' => match nums.first().copied().unwrap_or(0) {
            1 | 7 => Some(KeyCode::Home),
            3 => Some(KeyCode::Delete),
            4 | 8 => Some(KeyCode::End),
            5 => Some(KeyCode::PageUp),
            6 => Some(KeyCode::PageDown),
            11..=15 => Some(KeyCode::F((nums[0] - 10) as u8)),
            17..=21 => Some(KeyCode::F((nums[0] - 11) as u8)),
            23 | 24 => Some(KeyCode::F((nums[0] - 12) as u8)),
            _ => None,
        },
        _ => None,
    };
    if let Some(code) = code {
        out.push(key(code, modifiers));
    }
    Step::Consumed(consumed)
}

/// xterm modifier parameter: 1 + (shift=1 | alt=2 | ctrl=4).
fn decode_modifiers(m: u32) -> KeyModifiers {
    let bits = m.saturating_sub(1);
    KeyModifiers {
        shift: bits & 1 != 0,
        alt: bits & 2 != 0,
        ctrl: bits & 4 != 0,
    }
}

/// `params` is the part after `<`: `b;x;y`.
fn parse_sgr_mouse(params: &[u8], pressed: bool) -> Option<MouseEvent> {
    let s = std::str::from_utf8(params).ok()?;
    let mut it = s.split(';');
    let b: u32 = it.next()?.parse().ok()?;
    let x: u16 = it.next()?.parse().ok()?;
    let y: u16 = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None;
    }
    let kind = if b & 64 != 0 {
        // wheel: low bits 0 = up, 1 = down (press only)
        if !pressed {
            return None;
        }
        match b & 0b11 {
            0 => MouseKind::ScrollUp,
            1 => MouseKind::ScrollDown,
            _ => return None,
        }
    } else if b & 32 != 0 {
        return None; // motion / drag
    } else if b & 0b11 == 0 && pressed {
        MouseKind::LeftClick
    } else {
        return None; // releases, middle/right buttons
    };
    Some(MouseEvent {
        kind,
        column: x.saturating_sub(1),
        row: y.saturating_sub(1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: KeyCode) -> InputEvent {
        plain(code)
    }

    fn one(data: &[u8]) -> InputEvent {
        let evs = parse(data);
        assert_eq!(evs.len(), 1, "{data:?} -> {evs:?}");
        evs[0]
    }

    #[test]
    fn arrows_csi_and_ss3() {
        assert_eq!(one(b"\x1b[A"), k(KeyCode::Up));
        assert_eq!(one(b"\x1b[B"), k(KeyCode::Down));
        assert_eq!(one(b"\x1b[C"), k(KeyCode::Right));
        assert_eq!(one(b"\x1b[D"), k(KeyCode::Left));
        assert_eq!(one(b"\x1bOA"), k(KeyCode::Up));
        assert_eq!(one(b"\x1bOB"), k(KeyCode::Down));
        assert_eq!(one(b"\x1bOC"), k(KeyCode::Right));
        assert_eq!(one(b"\x1bOD"), k(KeyCode::Left));
    }

    #[test]
    fn home_end_page_delete_backtab() {
        assert_eq!(one(b"\x1b[H"), k(KeyCode::Home));
        assert_eq!(one(b"\x1b[F"), k(KeyCode::End));
        assert_eq!(one(b"\x1b[1~"), k(KeyCode::Home));
        assert_eq!(one(b"\x1b[4~"), k(KeyCode::End));
        assert_eq!(one(b"\x1bOH"), k(KeyCode::Home));
        assert_eq!(one(b"\x1bOF"), k(KeyCode::End));
        assert_eq!(one(b"\x1b[5~"), k(KeyCode::PageUp));
        assert_eq!(one(b"\x1b[6~"), k(KeyCode::PageDown));
        assert_eq!(one(b"\x1b[3~"), k(KeyCode::Delete));
        assert_eq!(
            one(b"\x1b[Z"),
            key(KeyCode::BackTab, KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn function_keys() {
        assert_eq!(one(b"\x1bOP"), k(KeyCode::F(1)));
        assert_eq!(one(b"\x1b[15~"), k(KeyCode::F(5)));
        assert_eq!(one(b"\x1b[17~"), k(KeyCode::F(6)));
        assert_eq!(one(b"\x1b[21~"), k(KeyCode::F(10)));
        assert_eq!(one(b"\x1b[24~"), k(KeyCode::F(12)));
    }

    #[test]
    fn basic_keys() {
        assert_eq!(one(b"\t"), k(KeyCode::Tab));
        assert_eq!(one(b"\r"), k(KeyCode::Enter));
        assert_eq!(one(b"\n"), k(KeyCode::Enter));
        assert_eq!(one(&[0x7f]), k(KeyCode::Backspace));
        assert_eq!(one(&[0x08]), k(KeyCode::Backspace));
        assert_eq!(one(b"\x1b"), k(KeyCode::Esc));
        assert_eq!(one(b" "), k(KeyCode::Char(' ')));
        assert_eq!(one(b"q"), k(KeyCode::Char('q')));
        assert_eq!(one(b"?"), k(KeyCode::Char('?')));
        assert_eq!(one(b"Q"), k(KeyCode::Char('Q')));
    }

    #[test]
    fn ctrl_keys() {
        assert_eq!(one(&[0x03]), InputEvent::Key(KeyEvent::ctrl('c')));
        assert_eq!(one(&[0x04]), InputEvent::Key(KeyEvent::ctrl('d')));
        assert_eq!(one(&[0x0c]), InputEvent::Key(KeyEvent::ctrl('l')));
        assert_eq!(one(&[0x01]), InputEvent::Key(KeyEvent::ctrl('a')));
        assert_eq!(one(&[0x1a]), InputEvent::Key(KeyEvent::ctrl('z')));
        assert_eq!(one(&[0x00]), InputEvent::Key(KeyEvent::ctrl(' ')));
        assert!(parse(&[0x1c]).is_empty());
    }

    #[test]
    fn utf8_and_turkish() {
        let s = "çğıöşüÇĞİÖŞÜ";
        let evs = parse(s.as_bytes());
        let expected: Vec<_> = s.chars().map(|c| k(KeyCode::Char(c))).collect();
        assert_eq!(evs, expected);
        assert_eq!(one("€".as_bytes()), k(KeyCode::Char('€')));
        assert_eq!(one("🦀".as_bytes()), k(KeyCode::Char('🦀')));
    }

    #[test]
    fn utf8_split_across_packets() {
        let bytes = "ş".as_bytes();
        let mut p = InputParser::new();
        assert!(p.feed(&bytes[..1]).is_empty());
        assert_eq!(p.feed(&bytes[1..]), vec![k(KeyCode::Char('ş'))]);
    }

    #[test]
    fn invalid_utf8_is_dropped_without_panic() {
        let evs = parse(&[0xff, 0x80, b'a', 0xc3, b'b']);
        assert_eq!(evs, vec![k(KeyCode::Char('a')), k(KeyCode::Char('b'))]);
    }

    #[test]
    fn mouse_click_and_wheel() {
        assert_eq!(
            one(b"\x1b[<0;10;5M"),
            InputEvent::Mouse(MouseEvent {
                kind: MouseKind::LeftClick,
                column: 9,
                row: 4
            })
        );
        assert_eq!(
            one(b"\x1b[<64;1;1M"),
            InputEvent::Mouse(MouseEvent {
                kind: MouseKind::ScrollUp,
                column: 0,
                row: 0
            })
        );
        assert_eq!(
            one(b"\x1b[<65;120;40M"),
            InputEvent::Mouse(MouseEvent {
                kind: MouseKind::ScrollDown,
                column: 119,
                row: 39
            })
        );
    }

    #[test]
    fn mouse_ignored_cases() {
        assert!(parse(b"\x1b[<0;10;5m").is_empty()); // release
        assert!(parse(b"\x1b[<2;10;5M").is_empty()); // right button
        assert!(parse(b"\x1b[<1;10;5M").is_empty()); // middle button
        assert!(parse(b"\x1b[<32;10;5M").is_empty()); // drag
        assert!(parse(b"\x1b[<35;10;5M").is_empty()); // motion
        assert!(parse(b"\x1b[<99;1M").is_empty());
    }

    #[test]
    fn mouse_with_modifier_bits_still_click() {
        // ctrl (16) + left button
        assert!(matches!(
            one(b"\x1b[<16;3;3M"),
            InputEvent::Mouse(MouseEvent {
                kind: MouseKind::LeftClick,
                ..
            })
        ));
    }

    #[test]
    fn modified_arrows() {
        assert_eq!(
            one(b"\x1b[1;5C"),
            key(KeyCode::Right, KeyModifiers::CTRL)
        );
        assert_eq!(one(b"\x1b[1;2A"), key(KeyCode::Up, KeyModifiers::SHIFT));
        assert_eq!(one(b"\x1b[1;3D"), key(KeyCode::Left, KeyModifiers::ALT));
    }

    #[test]
    fn alt_char() {
        assert_eq!(one(b"\x1bx"), key(KeyCode::Char('x'), KeyModifiers::ALT));
    }

    #[test]
    fn mixed_stream() {
        let evs = parse(b"j\x1b[B\x1b[<64;2;3Mk\r");
        assert_eq!(evs.len(), 5);
        assert_eq!(evs[0], k(KeyCode::Char('j')));
        assert_eq!(evs[1], k(KeyCode::Down));
        assert!(matches!(evs[2], InputEvent::Mouse(_)));
        assert_eq!(evs[3], k(KeyCode::Char('k')));
        assert_eq!(evs[4], k(KeyCode::Enter));
    }

    #[test]
    fn mixed_with_trailing_esc_and_enter() {
        let evs = parse(b"a\x1b\r");
        // ESC followed by CR => Alt+Enter
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0], k(KeyCode::Char('a')));
        assert_eq!(evs[1], key(KeyCode::Enter, KeyModifiers::ALT));
    }

    #[test]
    fn split_csi_across_packets() {
        let mut p = InputParser::new();
        assert!(p.feed(b"\x1b[").is_empty());
        assert_eq!(p.feed(b"A"), vec![k(KeyCode::Up)]);
        assert!(p.feed(b"\x1b[<0;1").is_empty());
        assert_eq!(p.feed(b"0;5M").len(), 1);
    }

    #[test]
    fn unknown_sequences_are_swallowed() {
        assert!(parse(b"\x1b[99X").is_empty());
        assert!(parse(b"\x1b[200~").is_empty());
        assert!(parse(b"\x1bOZ").is_empty());
        assert_eq!(parse(b"\x1b[99Xa"), vec![k(KeyCode::Char('a'))]);
    }

    #[test]
    fn malformed_csi_does_not_hang_or_panic() {
        let evs = parse(b"\x1b[\x01a");
        assert_eq!(evs.last(), Some(&k(KeyCode::Char('a'))));
        // Oversized garbage never accumulates unbounded
        let mut p = InputParser::new();
        let junk = vec![b'1'; 200];
        let mut data = b"\x1b[".to_vec();
        data.extend(junk);
        assert!(p.feed(&data).is_empty());
        assert_eq!(p.feed(b"q"), vec![k(KeyCode::Char('q'))]);
    }

    #[test]
    fn empty_input() {
        assert!(parse(b"").is_empty());
    }

    #[test]
    fn double_escape() {
        let evs = parse(b"\x1b\x1b");
        assert_eq!(evs, vec![k(KeyCode::Esc), k(KeyCode::Esc)]);
    }
}
