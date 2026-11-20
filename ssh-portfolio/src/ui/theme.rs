//! Brand palette (truecolor) and small rendering helpers.
//!
//! Colors are taken from `alttre.is-a.dev` (`style.css`, `lanyard.css`):
//! ThinkPad TrackPoint red on warm charcoal / cream paper.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding};

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub dark: bool,
    /// Canvas.
    pub bg: Color,
    /// Elevated panel.
    pub panel: Color,
    /// Highlighted row / chip.
    pub panel_hi: Color,
    pub ink: Color,
    pub ink2: Color,
    pub muted: Color,
    /// TrackPoint red.
    pub red: Color,
    pub rust: Color,
    pub sage: Color,
    pub amber: Color,
    pub terracotta: Color,
    pub border: Color,
    /// Text drawn on top of red / sage badges.
    pub on_accent: Color,
}

impl Theme {
    /// Obsidian dark — the default for SSH terminals.
    pub const fn dark() -> Self {
        Self {
            dark: true,
            bg: rgb(0x1E1B18),
            panel: rgb(0x27231F),
            panel_hi: rgb(0x302A24),
            ink: rgb(0xFAF6EE),
            ink2: rgb(0xE8DEC9),
            muted: rgb(0x8E8274),
            red: rgb(0xDA291C),
            rust: rgb(0xD46A43),
            sage: rgb(0x5E7060),
            amber: rgb(0xD47A3B),
            terracotta: rgb(0xB4472B),
            border: rgb(0x3E362E),
            on_accent: rgb(0xFAF6EE),
        }
    }

    /// Warm cream paper — just like the website.
    pub const fn cream() -> Self {
        Self {
            dark: false,
            bg: rgb(0xFAF6EE),
            panel: rgb(0xF3ECE0),
            panel_hi: rgb(0xE8DEC9),
            ink: rgb(0x1E1B18),
            ink2: rgb(0x585047),
            muted: rgb(0x8E8274),
            red: rgb(0xDA291C),
            rust: rgb(0xD46A43),
            sage: rgb(0x5E7060),
            amber: rgb(0xB4472B),
            terracotta: rgb(0xB4472B),
            border: rgb(0xD9C9B8),
            on_accent: rgb(0xFAF6EE),
        }
    }

    pub const fn new(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::cream() }
    }

    pub fn name(&self) -> &'static str {
        if self.dark { "OBSIDIAN" } else { "CREAM" }
    }

    // ── text styles ──
    pub fn canvas(&self) -> Style {
        Style::new().fg(self.ink).bg(self.bg)
    }
    pub fn text(&self) -> Style {
        Style::new().fg(self.ink)
    }
    pub fn text2(&self) -> Style {
        Style::new().fg(self.ink2)
    }
    pub fn dim(&self) -> Style {
        Style::new().fg(self.muted)
    }
    pub fn accent(&self) -> Style {
        Style::new().fg(self.red).add_modifier(Modifier::BOLD)
    }
    pub fn heading(&self) -> Style {
        Style::new().fg(self.ink).add_modifier(Modifier::BOLD)
    }
    pub fn rust_style(&self) -> Style {
        Style::new().fg(self.rust)
    }
    pub fn sage_text(&self) -> Style {
        // Sage reads fine on both canvases.
        Style::new().fg(if self.dark { rgb(0x8FAE92) } else { self.sage })
    }
    pub fn badge_red(&self) -> Style {
        Style::new()
            .fg(self.on_accent)
            .bg(self.red)
            .add_modifier(Modifier::BOLD)
    }
    pub fn badge_sage(&self) -> Style {
        Style::new().fg(self.on_accent).bg(self.sage)
    }
    pub fn badge_soft(&self) -> Style {
        Style::new().fg(self.ink2).bg(self.panel_hi)
    }
    pub fn selected_row(&self) -> Style {
        Style::new().bg(self.panel_hi)
    }
    pub fn key(&self) -> Style {
        Style::new().fg(self.red).add_modifier(Modifier::BOLD)
    }

    /// Rounded panel; focused panels get a TrackPoint-red border.
    pub fn panel_block<'a>(&self, title: impl Into<Line<'a>>, focused: bool) -> Block<'a> {
        let border = if focused { self.red } else { self.border };
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(border))
            .style(Style::new().fg(self.ink).bg(self.panel))
            .padding(Padding::horizontal(1))
            .title(title)
    }
}

/// ` text ` badge span.
pub fn badge<'a>(text: impl AsRef<str>, style: Style) -> Span<'a> {
    Span::styled(format!(" {} ", text.as_ref()), style)
}

/// Display width of `s` in terminal cells.
pub fn width(s: &str) -> usize {
    Line::from(s).width()
}

fn char_width(c: char) -> usize {
    let mut buf = [0u8; 4];
    Span::raw(&*c.encode_utf8(&mut buf)).width()
}

/// Truncate `s` to at most `w` cells, appending `…` when cut.
pub fn fit(s: &str, w: usize) -> String {
    if width(s) <= w {
        return s.to_string();
    }
    if w == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let cw = char_width(c);
        if used + cw + 1 > w {
            break;
        }
        out.push(c);
        used += cw;
    }
    out.push('…');
    out
}

/// Greedy word-wrap by display width. Never returns an empty vec.
pub fn wrap(text: &str, w: usize) -> Vec<String> {
    let w = w.max(1);
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        let mut cur_w = 0;
        for word in para.split_whitespace() {
            let mut word = word.to_string();
            let mut ww = width(&word);
            // Hard-break words longer than the line.
            while ww > w {
                if !cur.is_empty() {
                    lines.push(std::mem::take(&mut cur));
                    cur_w = 0;
                }
                let mut head = String::new();
                let mut used = 0;
                let mut rest = String::new();
                for c in word.chars() {
                    let cw = char_width(c);
                    if rest.is_empty() && used + cw <= w {
                        head.push(c);
                        used += cw;
                    } else {
                        rest.push(c);
                    }
                }
                if head.is_empty() {
                    // A single glyph wider than the line: emit it anyway.
                    let mut it = word.chars();
                    head.push(it.next().unwrap_or(' '));
                    rest = it.collect();
                }
                lines.push(head);
                word = rest;
                ww = width(&word);
            }
            if word.is_empty() {
                continue;
            }
            if cur.is_empty() {
                cur = word;
                cur_w = ww;
            } else if cur_w + 1 + ww <= w {
                cur.push(' ');
                cur.push_str(&word);
                cur_w += 1 + ww;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word;
                cur_w = ww;
            }
        }
        lines.push(cur);
    }
    lines
}

/// Wrap `text` and style every line.
pub fn wrapped_lines(text: &str, w: usize, style: Style) -> Vec<Line<'static>> {
    wrap(text, w)
        .into_iter()
        .map(|l| Line::from(Span::styled(l, style)))
        .collect()
}

/// Lay out badges across as many lines as needed.
pub fn badge_lines(items: &[&str], w: usize, style: Style, gap_style: Style) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut cur: Vec<Span<'static>> = Vec::new();
    let mut cur_w = 0;
    for item in items {
        let label = format!(" {item} ");
        let lw = width(&label);
        if cur_w > 0 && cur_w + 1 + lw > w {
            lines.push(Line::from(std::mem::take(&mut cur)));
            cur_w = 0;
        }
        if cur_w > 0 {
            cur.push(Span::styled(" ", gap_style));
            cur_w += 1;
        }
        cur.push(Span::styled(label, style));
        cur_w += lw;
    }
    if !cur.is_empty() {
        lines.push(Line::from(cur));
    }
    lines
}

/// `██████░░░░` style bar (`value` of `max`, `w` cells).
pub fn bar(value: u8, max: u8, w: usize, on: Style, off: Style) -> Vec<Span<'static>> {
    let max = max.max(1) as usize;
    let filled = (value as usize * w).div_ceil(max).min(w);
    vec![
        Span::styled("█".repeat(filled), on),
        Span::styled("░".repeat(w - filled), off),
    ]
}

/// A `w`×`h` rect centered in `area` (clamped to fit).
pub fn center(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

/// `true` if `(col,row)` lies inside `r`.
pub fn contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x.saturating_add(r.width) && row >= r.y && row < r.y.saturating_add(r.height)
}

/// Visible window start so `sel` stays on screen, keeping `prev` when possible.
pub fn scroll_window(prev: usize, sel: usize, visible: usize, total: usize) -> usize {
    let visible = visible.max(1);
    let mut off = prev.min(total.saturating_sub(visible));
    if sel < off {
        off = sel;
    } else if sel >= off + visible {
        off = sel + 1 - visible;
    }
    off.min(total.saturating_sub(visible))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_respects_width() {
        let lines = wrap("the quick brown fox jumps over the lazy dog", 10);
        assert!(lines.iter().all(|l| width(l) <= 10), "{lines:?}");
        assert_eq!(lines.join(" "), "the quick brown fox jumps over the lazy dog");
    }

    #[test]
    fn wrap_handles_long_words_and_turkish() {
        let lines = wrap("Sıfırdan geliştirme çğıöşü abcdefghijklmnop", 6);
        assert!(lines.iter().all(|l| width(l) <= 6), "{lines:?}");
        assert!(!wrap("", 5).is_empty());
    }

    #[test]
    fn fit_truncates_with_ellipsis() {
        assert_eq!(fit("hello", 10), "hello");
        assert_eq!(fit("hello world", 6), "hello…");
        assert_eq!(fit("x", 0), "");
    }

    #[test]
    fn scroll_window_keeps_selection_visible() {
        assert_eq!(scroll_window(0, 0, 5, 13), 0);
        assert_eq!(scroll_window(0, 7, 5, 13), 3);
        assert_eq!(scroll_window(3, 4, 5, 13), 3);
        assert_eq!(scroll_window(3, 2, 5, 13), 2);
    }

    #[test]
    fn palettes_match_brand() {
        assert_eq!(Theme::dark().bg, Color::Rgb(0x1E, 0x1B, 0x18));
        assert_eq!(Theme::dark().red, Color::Rgb(0xDA, 0x29, 0x1C));
        assert_eq!(Theme::cream().bg, Color::Rgb(0xFA, 0xF6, 0xEE));
    }
}
