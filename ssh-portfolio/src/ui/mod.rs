//! Ratatui portfolio UI for `ssh alttre.is-a.dev`.
//!
//! Public contract (used by `crate::ssh`):
//! * [`App::new`]
//! * [`App::handle_event`] — feed [`InputEvent`]s, get an [`AppAction`]
//! * [`render`] — draw the current state into a [`Frame`]
//!
//! Keys: `1`-`6`/`Tab`/`Shift+Tab` sections · `j`/`k`/arrows move · `Enter` open/flip ·
//! `Esc` close · `e` EN/TR · `t` theme · `?` help · `q`/`Ctrl+C`/`Ctrl+D` quit.

mod about;
mod competitions;
mod contact;
mod intro;
mod projects;
mod skills;
pub mod theme;

use std::cell::{Cell, RefCell};
use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};

use crate::data::{self, Bi, Lang};
use crate::events::{AppAction, InputEvent, KeyCode, KeyEvent, MouseEvent, MouseKind};
use theme::Theme;

/// Smallest terminal we try to draw a real layout in.
pub const MIN_WIDTH: u16 = 24;
pub const MIN_HEIGHT: u16 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Intro,
    Projects,
    Skills,
    Competitions,
    About,
    Contact,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Intro,
        Tab::Projects,
        Tab::Skills,
        Tab::Competitions,
        Tab::About,
        Tab::Contact,
    ];

    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Tab {
        Tab::ALL[i % Tab::ALL.len()]
    }

    pub fn label(self) -> Bi {
        match self {
            Tab::Intro => Bi::new("Intro", "Giriş"),
            Tab::Projects => Bi::new("Projects", "Projeler"),
            Tab::Skills => Bi::new("Skills", "Yetenekler"),
            Tab::Competitions => Bi::new("Competitions", "Yarışmalar"),
            Tab::About => Bi::new("About", "Hakkında"),
            Tab::Contact => Bi::new("Contact", "İletişim"),
        }
    }
}

/// Clickable regions recorded during `render` (mouse hit-testing).
#[derive(Debug, Default, Clone)]
pub(crate) struct Hits {
    pub tabs: Vec<Rect>,
    /// `(region, item index)` for the active tab's clickable items.
    pub items: Vec<(Rect, usize)>,
}

#[derive(Debug)]
pub struct App {
    tab: Tab,
    lang: Lang,
    dark: bool,
    help: bool,
    started: Instant,
    tick: u64,

    // intro typewriter
    type_line: usize,
    type_pos: usize,
    type_hold: u16,

    // projects
    proj_filter: usize,
    proj_sel: usize,
    proj_open: bool,
    proj_scroll: u16,
    proj_offset: Cell<usize>,

    // skills
    skill_cat: usize,

    // competitions
    comp_sel: usize,
    comp_flipped: [bool; 3],

    // about / dev logs
    about_sel: usize,
    about_open: bool,
    about_scroll: u16,

    // contact
    contact_sel: usize,

    // render feedback
    size: Cell<(u16, u16)>,
    hits: RefCell<Hits>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nav {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

pub(crate) fn nav_of(k: &KeyEvent) -> Option<Nav> {
    if k.modifiers.ctrl || k.modifiers.alt {
        return None;
    }
    match k.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Nav::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Nav::Down),
        KeyCode::PageUp => Some(Nav::PageUp),
        KeyCode::PageDown => Some(Nav::PageDown),
        KeyCode::Home | KeyCode::Char('g') => Some(Nav::Home),
        KeyCode::End | KeyCode::Char('G') => Some(Nav::End),
        _ => None,
    }
}

pub(crate) fn is_left(k: &KeyEvent) -> bool {
    !k.modifiers.ctrl && matches!(k.code, KeyCode::Left | KeyCode::Char('h'))
}

pub(crate) fn is_right(k: &KeyEvent) -> bool {
    !k.modifiers.ctrl && matches!(k.code, KeyCode::Right | KeyCode::Char('l'))
}

pub(crate) fn is_activate(k: &KeyEvent) -> bool {
    matches!(k.code, KeyCode::Enter | KeyCode::Char(' '))
}

/// Apply `nav` to a selection in `0..len`.
pub(crate) fn step(sel: usize, len: usize, nav: Nav, page: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    match nav {
        Nav::Up => sel.saturating_sub(1),
        Nav::Down => (sel + 1).min(last),
        Nav::PageUp => sel.saturating_sub(page.max(1)),
        Nav::PageDown => (sel + page.max(1)).min(last),
        Nav::Home => 0,
        Nav::End => last,
    }
    .min(last)
}

impl App {
    pub fn new() -> Self {
        Self {
            tab: Tab::Intro,
            lang: Lang::En,
            dark: true,
            help: false,
            started: Instant::now(),
            tick: 0,
            type_line: 0,
            type_pos: 0,
            type_hold: 0,
            proj_filter: 0,
            proj_sel: 0,
            proj_open: false,
            proj_scroll: 0,
            proj_offset: Cell::new(0),
            skill_cat: 0,
            comp_sel: 0,
            comp_flipped: [false; 3],
            about_sel: 0,
            about_open: false,
            about_scroll: 0,
            contact_sel: 0,
            size: Cell::new((80, 24)),
            hits: RefCell::new(Hits::default()),
        }
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::new(self.dark)
    }

    pub fn handle_event(&mut self, event: InputEvent) -> AppAction {
        match event {
            InputEvent::Key(k) => self.on_key(k),
            InputEvent::Mouse(m) => {
                self.on_mouse(m);
                AppAction::Continue
            }
            InputEvent::Resize(w, h) => {
                self.size.set((w, h));
                AppAction::Continue
            }
            InputEvent::Tick => {
                self.on_tick();
                AppAction::Continue
            }
        }
    }

    // ───────────────────────────── ticking ─────────────────────────────

    fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let n = data::INTRO_LINES[self.type_line].get(self.lang).chars().count();
        if self.type_pos < n {
            self.type_pos += 1;
        } else {
            self.type_hold += 1;
            if self.type_hold >= 24 {
                self.type_hold = 0;
                self.type_pos = 0;
                self.type_line = (self.type_line + 1) % data::INTRO_LINES.len();
            }
        }
    }

    pub(crate) fn typed_text(&self) -> String {
        data::INTRO_LINES[self.type_line]
            .get(self.lang)
            .chars()
            .take(self.type_pos)
            .collect()
    }

    pub(crate) fn uptime_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    // ────────────────────────────── keyboard ──────────────────────────────

    fn goto(&mut self, tab: Tab) {
        self.tab = tab;
        self.proj_open = false;
        self.about_open = false;
    }

    fn on_key(&mut self, k: KeyEvent) -> AppAction {
        if k.modifiers.ctrl {
            return match k.code {
                KeyCode::Char('c') | KeyCode::Char('d') => AppAction::Quit,
                _ => AppAction::Continue,
            };
        }
        if k.modifiers.alt {
            return AppAction::Continue;
        }

        // Keys that work everywhere (also over the help modal).
        match k.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return AppAction::Quit,
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.dark = !self.dark;
                return AppAction::Continue;
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.lang = self.lang.toggle();
                self.type_pos = 0;
                self.type_hold = 0;
                return AppAction::Continue;
            }
            _ => {}
        }

        if self.help {
            if matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?')) {
                self.help = false;
            }
            return AppAction::Continue;
        }

        match k.code {
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.help = true;
                return AppAction::Continue;
            }
            KeyCode::Char(c @ '1'..='6') => {
                self.goto(Tab::from_index(c as usize - '1' as usize));
                return AppAction::Continue;
            }
            KeyCode::Tab => {
                self.goto(Tab::from_index(self.tab.index() + 1));
                return AppAction::Continue;
            }
            KeyCode::BackTab => {
                self.goto(Tab::from_index(self.tab.index() + Tab::ALL.len() - 1));
                return AppAction::Continue;
            }
            _ => {}
        }

        if self.section_key(&k) {
            return AppAction::Continue;
        }

        // Fallback: Left/Right cycle sections.
        if is_left(&k) {
            self.goto(Tab::from_index(self.tab.index() + Tab::ALL.len() - 1));
        } else if is_right(&k) {
            self.goto(Tab::from_index(self.tab.index() + 1));
        }
        AppAction::Continue
    }

    fn section_key(&mut self, k: &KeyEvent) -> bool {
        match self.tab {
            Tab::Intro => intro::key(self, k),
            Tab::Projects => projects::key(self, k),
            Tab::Skills => skills::key(self, k),
            Tab::Competitions => competitions::key(self, k),
            Tab::About => about::key(self, k),
            Tab::Contact => contact::key(self, k),
        }
    }

    // ─────────────────────────────── mouse ───────────────────────────────

    fn on_mouse(&mut self, m: MouseEvent) {
        match m.kind {
            MouseKind::LeftClick => {
                if self.help {
                    self.help = false;
                    return;
                }
                let tab_hit = self
                    .hits
                    .borrow()
                    .tabs
                    .iter()
                    .position(|r| theme::contains(*r, m.column, m.row));
                if let Some(i) = tab_hit {
                    self.goto(Tab::from_index(i));
                    return;
                }
                let item = self
                    .hits
                    .borrow()
                    .items
                    .iter()
                    .find(|(r, _)| theme::contains(*r, m.column, m.row))
                    .map(|(_, i)| *i);
                if let Some(i) = item {
                    match self.tab {
                        Tab::Intro => {}
                        Tab::Projects => projects::click(self, i),
                        Tab::Skills => skills::click(self, i),
                        Tab::Competitions => competitions::click(self, i),
                        Tab::About => about::click(self, i),
                        Tab::Contact => contact::click(self, i),
                    }
                }
            }
            MouseKind::ScrollUp => self.on_wheel(-1),
            MouseKind::ScrollDown => self.on_wheel(1),
        }
    }

    fn on_wheel(&mut self, delta: i32) {
        if self.help {
            return;
        }
        match self.tab {
            Tab::Intro => {}
            Tab::Projects => projects::scroll(self, delta),
            Tab::Skills => skills::scroll(self, delta),
            Tab::Competitions => competitions::scroll(self, delta),
            Tab::About => about::scroll(self, delta),
            Tab::Contact => contact::scroll(self, delta),
        }
    }
}

// ═════════════════════════════════ rendering ═════════════════════════════════

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    app.size.set((area.width, area.height));
    *app.hits.borrow_mut() = Hits::default();
    let th = app.theme();

    frame.render_widget(Block::new().style(th.canvas()), area);

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area, app, &th);
        return;
    }

    let header_h = if area.height >= 20 { 3 } else { 1 };
    let [head, body, foot] = Layout::vertical([
        Constraint::Length(header_h),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);

    render_header(frame, head, app, &th);

    match app.tab {
        Tab::Intro => intro::render(frame, body, app),
        Tab::Projects => projects::render(frame, body, app),
        Tab::Skills => skills::render(frame, body, app),
        Tab::Competitions => competitions::render(frame, body, app),
        Tab::About => about::render(frame, body, app),
        Tab::Contact => contact::render(frame, body, app),
    }

    render_footer(frame, foot, app, &th);

    if app.help {
        render_help(frame, area, app, &th);
    }
}

fn render_too_small(frame: &mut Frame, area: Rect, app: &App, th: &Theme) {
    let lang = app.lang;
    let lines = vec![
        Line::from(Span::styled("A_", th.badge_red())),
        Line::from(Span::styled(
            if lang == Lang::En { "Resize terminal" } else { "Terminali büyütün" },
            th.accent(),
        )),
        Line::from(Span::styled(
            format!("{}x{} < {}x{}", area.width, area.height, MIN_WIDTH, MIN_HEIGHT),
            th.dim(),
        )),
    ];
    let rect = theme::center(area, area.width, (lines.len() as u16).min(area.height));
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).style(th.canvas()),
        rect,
    );
}

/// Tab strip labels for `avail` cells: full labels if they fit, otherwise
/// numbers with only the active tab spelled out.
fn tab_cells(lang: Lang, active: Tab, avail: usize) -> Vec<String> {
    let full: Vec<String> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| format!(" {} {} ", i + 1, t.label().get(lang)))
        .collect();
    let total: usize = full.iter().map(|s| theme::width(s)).sum::<usize>() + full.len() - 1;
    if total <= avail {
        return full;
    }
    Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if *t == active {
                format!(" {} {} ", i + 1, t.label().get(lang))
            } else {
                format!(" {} ", i + 1)
            }
        })
        .collect()
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, th: &Theme) {
    let lang = app.lang;
    let (row_y, x0, avail) = if area.height >= 3 {
        let pulse = if (app.tick / 5).is_multiple_of(2) { "●" } else { "○" };
        let up = app.uptime_secs();
        let right = Line::from(vec![
            Span::styled(format!(" {pulse} "), Style::new().fg(th.red)),
            Span::styled("LIVE", th.accent()),
            Span::styled(
                format!(" {:02}:{:02} · {} · {} ", up / 60 % 100, up % 60, lang.code(), th.name()),
                th.dim(),
            ),
        ])
        .right_aligned();
        let left = Line::from(vec![
            Span::raw(" "),
            Span::styled(" A_ ", th.badge_red()),
            Span::styled(format!(" {} ", data::STAMP), th.heading()),
        ]);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(th.border))
            .style(th.canvas())
            .title(left)
            .title(right);
        frame.render_widget(block, area);
        let avail = (area.width as usize).saturating_sub(4);
        (area.y + 1, area.x + 2, avail)
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(" A_ ", th.badge_red()))),
            Rect::new(area.x, area.y, area.width.min(4), 1),
        );
        let avail = (area.width as usize).saturating_sub(5);
        (area.y, area.x + 5, avail)
    };

    let cells = tab_cells(lang, app.tab, avail);
    let mut x = x0;
    let mut rects = Vec::new();
    let limit = area.x + area.width;
    for (i, cell) in cells.iter().enumerate() {
        if x >= limit {
            // Off-screen tabs get an empty rect so indexes stay aligned.
            rects.push(Rect::new(limit.saturating_sub(1), row_y, 0, 1));
            continue;
        }
        let w = (theme::width(cell) as u16).min(limit - x);
        let rect = Rect::new(x, row_y, w, 1);
        let style = if Tab::from_index(i) == app.tab {
            th.badge_red()
        } else {
            Style::new().fg(th.ink2).bg(th.bg)
        };
        frame.render_widget(Paragraph::new(Span::styled(cell.clone(), style)), rect);
        rects.push(rect);
        x = x.saturating_add(w + 1);
    }
    // Make the whole header strip clickable on the tab row's top/bottom border too.
    if area.height >= 3 {
        let extra: Vec<Rect> = rects
            .iter()
            .map(|r| Rect::new(r.x, area.y, r.width, area.height))
            .collect();
        app.hits.borrow_mut().tabs = extra;
    } else {
        app.hits.borrow_mut().tabs = rects;
    }
}

fn context_hint(app: &App) -> Bi {
    match app.tab {
        Tab::Intro => Bi::new("Enter: explore projects", "Enter: projelere git"),
        Tab::Projects if app.proj_open => {
            Bi::new("Esc back · ←→ prev/next · j/k scroll", "Esc geri · ←→ önceki/sonraki · j/k kaydır")
        }
        Tab::Projects => Bi::new("j/k select · ←→/f filter · Enter detail", "j/k seç · ←→/f filtre · Enter detay"),
        Tab::Skills => Bi::new("j/k category", "j/k kategori"),
        Tab::Competitions => Bi::new("←→ card · Enter flip", "←→ kart · Enter çevir"),
        Tab::About if app.about_open => {
            Bi::new("Esc back · n/p log · j/k scroll", "Esc geri · n/p yazı · j/k kaydır")
        }
        Tab::About => Bi::new("j/k log · Enter read · PgUp/PgDn", "j/k yazı · Enter oku · PgUp/PgDn"),
        Tab::Contact => Bi::new("j/k select", "j/k seç"),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App, th: &Theme) {
    let lang = app.lang;
    let bar = Style::new().fg(th.ink2).bg(th.panel);
    let key = th.key().bg(th.panel);
    let tr = lang == Lang::Tr;

    let mut right: Vec<Span> = Vec::new();
    for (k, label) in [
        ("e", if tr { "TR/EN" } else { "EN/TR" }),
        ("t", if tr { "Tema" } else { "Theme" }),
        ("?", if tr { "Yardım" } else { "Help" }),
        ("q", if tr { "Çık" } else { "Quit" }),
    ] {
        right.push(Span::styled(format!("[{k}]"), key));
        right.push(Span::styled(format!("{label} "), bar));
    }
    let right_w: usize = right.iter().map(|s| s.width()).sum();

    let left_text = format!(" {} ", context_hint(app).get(lang));
    let left_w = theme::width(&left_text);
    let avail = area.width as usize;

    let mut spans: Vec<Span> = Vec::new();
    if left_w + right_w < avail {
        spans.push(Span::styled(left_text, th.dim().bg(th.panel)));
        spans.push(Span::styled(" ".repeat(avail - left_w - right_w), bar));
        spans.extend(right);
    } else if right_w <= avail {
        spans.push(Span::styled(" ".repeat(avail - right_w), bar));
        spans.extend(right);
    } else {
        spans.push(Span::styled("[?]", key));
        spans.push(Span::styled(" [q]", key));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).style(bar), area);
}

fn render_help(frame: &mut Frame, area: Rect, app: &App, th: &Theme) {
    let tr = app.lang == Lang::Tr;
    let rows: Vec<(&str, &str, &str)> = vec![
        ("1 … 6", "Jump to section", "Bölüme git"),
        ("Tab / Shift+Tab", "Next / previous section", "Sonraki / önceki bölüm"),
        ("← → / h l", "Filter · card · section", "Filtre · kart · bölüm"),
        ("↑ ↓ / j k", "Move selection / scroll", "Seçimi taşı / kaydır"),
        ("PgUp PgDn · g G", "Page · top / bottom", "Sayfa · baş / son"),
        ("Enter / Space", "Open detail · flip card", "Detay aç · kartı çevir"),
        ("Esc", "Close detail / this help", "Detayı / yardımı kapat"),
        ("f", "Projects: next filter", "Projeler: sonraki filtre"),
        ("n / p", "Dev logs: next / previous", "Yazılar: sonraki / önceki"),
        ("e", "Switch language EN ⇄ TR", "Dil değiştir EN ⇄ TR"),
        ("t", "Theme: Obsidian ⇄ Cream", "Tema: Obsidian ⇄ Krem"),
        ("?", "Toggle this help", "Bu yardımı aç/kapat"),
        ("q · Ctrl+C · Ctrl+D", "Quit", "Çıkış"),
        ("Mouse", "Click tabs/items · wheel scrolls", "Sekme/öğe tıkla · tekerlek kaydırır"),
    ];
    let key_w = rows.iter().map(|r| theme::width(r.0)).max().unwrap_or(0);
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        if tr { "KLAVYE & FARE" } else { "KEYBOARD & MOUSE" },
        th.accent(),
    )));
    lines.push(Line::raw(""));
    for (k, en, trs) in &rows {
        let pad = " ".repeat(key_w - theme::width(k) + 2);
        lines.push(Line::from(vec![
            Span::styled(format!(" {k} "), th.badge_soft()),
            Span::raw(pad),
            Span::styled(if tr { *trs } else { *en }, th.text()),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        if tr { "Esc / ? ile kapat" } else { "Press Esc or ? to close" },
        th.dim(),
    )));

    let w = (key_w as u16 + 40).min(area.width.saturating_sub(2)).max(1);
    let h = (lines.len() as u16 + 2).min(area.height.saturating_sub(2)).max(1);
    let rect = theme::center(area, w, h);
    frame.render_widget(Clear, rect);
    let title = Line::from(vec![
        Span::styled(" ? ", th.badge_red()),
        Span::styled(if tr { " Yardım " } else { " Help " }, th.heading()),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

// ═══════════════════════════════════ tests ═══════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::KeyModifiers;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn key(c: char) -> InputEvent {
        InputEvent::Key(KeyEvent::plain(KeyCode::Char(c)))
    }
    fn code(c: KeyCode) -> InputEvent {
        InputEvent::Key(KeyEvent::plain(c))
    }

    fn draw(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, app)).unwrap();
        let buf = term.backend().buffer();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn app_on(tab: Tab) -> App {
        let mut a = App::new();
        a.goto(tab);
        a
    }

    #[test]
    fn renders_every_tab_at_all_sizes() {
        for &(w, h) in &[(120u16, 36u16), (80, 24), (30, 12)] {
            for tab in Tab::ALL {
                let mut app = app_on(tab);
                for _ in 0..40 {
                    app.handle_event(InputEvent::Tick);
                }
                let s = draw(&app, w, h);
                assert!(!s.trim().is_empty(), "{tab:?} {w}x{h}");
            }
        }
    }

    #[test]
    fn never_panics_on_any_size_state_or_language() {
        for tab in Tab::ALL {
            for (lang_tr, dark) in [(false, true), (true, false)] {
                let mut app = app_on(tab);
                if lang_tr {
                    app.handle_event(key('e'));
                }
                if !dark {
                    app.handle_event(key('t'));
                }
                for _ in 0..30 {
                    app.handle_event(InputEvent::Tick);
                }
                for open in [false, true] {
                    if open {
                        app.handle_event(code(KeyCode::Enter));
                        app.handle_event(code(KeyCode::Down));
                    }
                    for help in [false, true] {
                        if help != app.help {
                            app.handle_event(key('?'));
                        }
                        for w in (0u16..=140).step_by(7).chain([1, 2, 23, 24, 25, 71, 72, 73]) {
                            for h in (0u16..=45).step_by(3).chain([1, 6, 7, 8, 19, 20, 21]) {
                                draw(&app, w, h);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn quit_keys() {
        let mut app = App::new();
        assert_eq!(app.handle_event(key('q')), AppAction::Quit);
        assert_eq!(app.handle_event(InputEvent::Key(KeyEvent::ctrl('c'))), AppAction::Quit);
        assert_eq!(app.handle_event(InputEvent::Key(KeyEvent::ctrl('d'))), AppAction::Quit);
        assert_eq!(app.handle_event(key('x')), AppAction::Continue);
        assert_eq!(app.handle_event(InputEvent::Key(KeyEvent::ctrl('x'))), AppAction::Continue);
        // q also quits while help is open; Esc only closes it.
        app.handle_event(key('?'));
        assert_eq!(app.handle_event(code(KeyCode::Esc)), AppAction::Continue);
        assert!(!app.help);
        app.handle_event(key('?'));
        assert_eq!(app.handle_event(key('q')), AppAction::Quit);
    }

    #[test]
    fn number_keys_and_tab_cycle_sections() {
        let mut app = App::new();
        for (i, tab) in Tab::ALL.iter().enumerate() {
            app.handle_event(key(char::from_digit(i as u32 + 1, 10).unwrap()));
            assert_eq!(app.tab, *tab);
        }
        app.handle_event(code(KeyCode::Tab));
        assert_eq!(app.tab, Tab::Intro);
        app.handle_event(code(KeyCode::BackTab));
        assert_eq!(app.tab, Tab::Contact);
        // Left/Right fall through to section cycling on tabs without a custom meaning.
        app.handle_event(code(KeyCode::Left));
        assert_eq!(app.tab, Tab::About);
        app.handle_event(code(KeyCode::Right));
        assert_eq!(app.tab, Tab::Contact);
    }

    #[test]
    fn theme_toggle_changes_canvas_color() {
        let mut app = App::new();
        assert!(app.dark);
        app.handle_event(key('t'));
        assert!(!app.dark);
        let mut term = Terminal::new(TestBackend::new(60, 20)).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        assert_eq!(term.backend().buffer()[(0, 19)].bg, Theme::cream().panel);
        assert_eq!(term.backend().buffer()[(0, 0)].bg, Theme::cream().bg);
        app.handle_event(key('t'));
        term.draw(|f| render(f, &app)).unwrap();
        assert_eq!(term.backend().buffer()[(0, 0)].bg, Theme::dark().bg);
    }

    #[test]
    fn language_toggle_switches_labels() {
        let mut app = App::new();
        let en = draw(&app, 120, 36);
        assert!(en.contains("Projects") && en.contains("Competitions"));
        assert!(en.contains("EN/TR") && en.contains("Theme"));
        app.handle_event(key('e'));
        assert_eq!(app.lang, Lang::Tr);
        let tr = draw(&app, 120, 36);
        assert!(tr.contains("Projeler") && tr.contains("Yarışmalar"));
        assert!(tr.contains("TR/EN") && tr.contains("Tema"));
        assert!(!tr.contains("Projects"));
        // Turkish content in a section body too.
        app.handle_event(key('5'));
        assert!(draw(&app, 120, 36).contains("Sıfırdan Geliştirme"));
        app.handle_event(key('e'));
        assert!(draw(&app, 120, 36).contains("From-Scratch Development"));
    }

    #[test]
    fn project_navigation_and_filter() {
        let mut app = App::new();
        app.handle_event(key('2'));
        let s = draw(&app, 120, 36);
        assert!(s.contains("NexusAI-PhoneAgent"));
        assert!(s.contains("All (13)"));
        app.handle_event(key('j'));
        app.handle_event(code(KeyCode::Down));
        assert_eq!(app.proj_sel, 2);
        // Filter: Right -> AI (5 projects).
        app.handle_event(code(KeyCode::Right));
        assert_eq!(app.proj_filter, 1);
        assert_eq!(app.proj_sel, 0);
        assert!(draw(&app, 120, 36).contains("AI (5)"));
        app.handle_event(key('f'));
        assert_eq!(app.proj_filter, 2);
        app.handle_event(key('f'));
        app.handle_event(key('f'));
        assert_eq!(app.proj_filter, 0);
        // Walk to the very end of the unfiltered list.
        app.handle_event(key('G'));
        assert_eq!(app.proj_sel, 12);
        let s = draw(&app, 120, 36);
        assert!(s.contains("İş-Bul"));
        // Enter toggles expanded detail, Esc closes it.
        app.handle_event(code(KeyCode::Enter));
        assert!(app.proj_open);
        assert!(draw(&app, 80, 24).contains("is-bul.vercel.app"));
        app.handle_event(code(KeyCode::Esc));
        assert!(!app.proj_open);
        // Narrow terminals: single list; Enter opens the detail pane.
        let narrow = draw(&app, 50, 20);
        assert!(!narrow.contains("is-bul.vercel.app"));
        app.handle_event(code(KeyCode::Enter));
        assert!(draw(&app, 50, 20).contains("is-bul.vercel.app"));
    }

    #[test]
    fn competition_card_flips() {
        let mut app = App::new();
        app.handle_event(key('4'));
        let front = draw(&app, 120, 36);
        assert!(front.contains("SAHADAN NOTLAR / 01—03"));
        assert!(front.contains("ALKÜ Hackathon"));
        assert!(front.contains("Fibonacci Robot Olimpiyatları"));
        assert!(front.contains("MEB Robot"));
        assert!(front.contains("36h Sprint"));
        assert!(!front.contains("QTR"));
        app.handle_event(code(KeyCode::Right));
        assert_eq!(app.comp_sel, 1);
        app.handle_event(code(KeyCode::Enter));
        assert!(app.comp_flipped[1]);
        let back = draw(&app, 120, 36);
        assert!(back.contains("QTR") && back.contains("MPU6050") && back.contains("500 g"));
        app.handle_event(key(' '));
        assert!(!app.comp_flipped[1]);
    }

    #[test]
    fn help_overlay_opens_and_closes() {
        let mut app = App::new();
        app.handle_event(key('?'));
        assert!(app.help);
        let s = draw(&app, 100, 30);
        assert!(s.contains("KEYBOARD & MOUSE") && s.contains("Ctrl+C"));
        // Section keys are inert while help is open.
        app.handle_event(key('3'));
        assert_eq!(app.tab, Tab::Intro);
        app.handle_event(key('?'));
        assert!(!app.help);
        app.handle_event(key('?'));
        app.handle_event(code(KeyCode::Esc));
        assert!(!app.help);
        app.handle_event(key('e'));
        app.handle_event(key('?'));
        assert!(draw(&app, 100, 30).contains("KLAVYE & FARE"));
    }

    #[test]
    fn typewriter_advances_and_cycles() {
        let mut app = App::new();
        assert_eq!(app.typed_text(), "");
        for _ in 0..5 {
            app.handle_event(InputEvent::Tick);
        }
        assert_eq!(app.typed_text(), "Hello");
        for _ in 0..80 {
            app.handle_event(InputEvent::Tick);
        }
        assert!(app.type_line > 0);
        // Language switch restarts the line in Turkish.
        app.handle_event(key('e'));
        app.handle_event(InputEvent::Tick);
        assert_eq!(app.type_pos, 1);
    }

    #[test]
    fn intro_shows_brand_stats_and_cta() {
        let mut app = App::new();
        for _ in 0..200 {
            app.handle_event(InputEvent::Tick);
        }
        let s = draw(&app, 120, 36);
        assert!(s.contains("ALTTRE // 2026"));
        assert!(s.contains("13 Projects") && s.contains("22 Skills") && s.contains("4 Hackathons"));
        assert!(s.contains("Explore Projects"));
        assert!(s.contains("◉ ◉"));
        // Enter jumps to projects.
        app.handle_event(code(KeyCode::Enter));
        assert_eq!(app.tab, Tab::Projects);
    }

    #[test]
    fn skills_lists_categories_and_links() {
        let mut app = App::new();
        app.handle_event(key('3'));
        let s = draw(&app, 120, 36);
        assert!(s.contains("Programming Languages") && s.contains("Hardware & Lab"));
        assert!(s.contains("Python") && s.contains("Flask"));
        app.handle_event(key('j'));
        app.handle_event(key('j'));
        app.handle_event(key('j'));
        let s = draw(&app, 120, 36);
        assert!(s.contains("Arduino ↔ Circuit Design") || s.contains("Circuit Design ↔ Soldering"));
    }

    #[test]
    fn about_dev_logs_reader() {
        let mut app = App::new();
        app.handle_event(key('5'));
        let s = draw(&app, 120, 36);
        assert!(s.contains("Ali Turan") && s.contains("Nebula Team"));
        assert!(s.contains("36-hour AI Agent"));
        app.handle_event(key('G'));
        assert_eq!(app.about_sel, 4);
        app.handle_event(code(KeyCode::Enter));
        assert!(app.about_open);
        assert!(draw(&app, 120, 36).contains("without a framework") || draw(&app, 120, 36).contains("framework"));
        app.handle_event(key('p'));
        assert_eq!(app.about_sel, 3);
        app.handle_event(key('n'));
        assert_eq!(app.about_sel, 4);
        app.handle_event(code(KeyCode::Esc));
        assert!(!app.about_open);
    }

    #[test]
    fn contact_cards_listed() {
        let mut app = App::new();
        app.handle_event(key('6'));
        let s = draw(&app, 120, 36);
        for needle in [
            "alttre.is-a.dev",
            "ali-turan-@outlook.com",
            "github.com/Alttrest",
            "devpost.com/Alttrest",
            "LinkedIn",
            "@Alttre",
            "Discord",
            "Istanbul",
        ] {
            assert!(s.contains(needle), "missing {needle}");
        }
    }

    #[test]
    fn mouse_clicks_switch_tabs_and_scroll_moves_selection() {
        let mut app = App::new();
        let _ = draw(&app, 120, 36);
        // Click the "Projects" tab (second tab) on the tab row.
        let rect = app.hits.borrow().tabs[1];
        app.handle_event(InputEvent::Mouse(MouseEvent {
            kind: MouseKind::LeftClick,
            column: rect.x + 1,
            row: rect.y + 1,
        }));
        assert_eq!(app.tab, Tab::Projects);
        let _ = draw(&app, 120, 36);
        app.handle_event(InputEvent::Mouse(MouseEvent { kind: MouseKind::ScrollDown, column: 5, row: 12 }));
        assert_eq!(app.proj_sel, 1);
        app.handle_event(InputEvent::Mouse(MouseEvent { kind: MouseKind::ScrollUp, column: 5, row: 12 }));
        assert_eq!(app.proj_sel, 0);
        // Clicking a list row selects it.
        let (r, idx) = app.hits.borrow().items[3];
        app.handle_event(InputEvent::Mouse(MouseEvent { kind: MouseKind::LeftClick, column: r.x + 1, row: r.y }));
        assert_eq!(app.proj_sel, idx);
    }

    #[test]
    fn tiny_terminal_shows_resize_notice() {
        let app = App::new();
        assert!(draw(&app, 20, 10).contains("Resize terminal"));
        assert!(draw(&app, 40, 5).contains("Resize terminal"));
        assert!(!draw(&app, 40, 15).contains("Resize terminal"));
        // Degenerate sizes must not panic.
        for (w, h) in [(0, 0), (1, 1), (3, 40), (200, 2)] {
            draw(&app, w, h);
        }
    }

    #[test]
    fn resize_event_and_modifier_keys_are_safe() {
        let mut app = App::new();
        assert_eq!(app.handle_event(InputEvent::Resize(100, 40)), AppAction::Continue);
        let alt = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
        assert_eq!(app.handle_event(InputEvent::Key(alt)), AppAction::Continue);
        for c in [KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown, KeyCode::Delete, KeyCode::Backspace, KeyCode::F(5)] {
            for tab in Tab::ALL {
                app.goto(tab);
                assert_eq!(app.handle_event(code(c)), AppAction::Continue);
            }
        }
    }
}
