//! Skills tab: 5 categories, 22 skills with focus bars and cross-link graph relations.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::{App, nav_of, step, theme};
use crate::data::{self, Lang};
use crate::events::KeyEvent;

pub(crate) fn key(app: &mut App, k: &KeyEvent) -> bool {
    if let Some(nav) = nav_of(k) {
        app.skill_cat = step(app.skill_cat, data::SKILL_CATEGORIES.len(), nav, 2);
        return true;
    }
    false
}

pub(crate) fn click(app: &mut App, idx: usize) {
    if idx < data::SKILL_CATEGORIES.len() {
        app.skill_cat = idx;
    }
}

pub(crate) fn scroll(app: &mut App, delta: i32) {
    let len = data::SKILL_CATEGORIES.len();
    let next = (app.skill_cat as isize + delta as isize).clamp(0, len as isize - 1);
    app.skill_cat = next as usize;
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let th = app.theme();
    let lang = app.lang;
    let sel = app.skill_cat.min(data::SKILL_CATEGORIES.len() - 1);

    if area.width < 68 {
        let [top, bottom] = Layout::vertical([
            Constraint::Length((data::SKILL_CATEGORIES.len() as u16 + 2).min(area.height / 2 + 1)),
            Constraint::Min(1),
        ])
        .areas(area);
        render_categories(frame, top, sel, app, &th);
        render_skills(frame, bottom, sel, lang, &th);
    } else {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(34),
            Constraint::Percentage(66),
        ])
        .areas(area);
        render_categories(frame, left, sel, app, &th);
        render_skills(frame, right, sel, lang, &th);
    }
}

fn render_categories(
    frame: &mut Frame,
    area: Rect,
    sel: usize,
    app: &App,
    th: &theme::Theme,
) {
    let lang = app.lang;
    let title = Line::from(vec![
        Span::styled(
            if lang == Lang::En { " Categories " } else { " Kategoriler " },
            th.heading(),
        ),
        Span::styled(format!("({}) ", data::skill_count()), th.dim()),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut y = inner.y;
    let mut hits = app.hits.borrow_mut();

    // Root node header if space permits
    if inner.height >= (data::SKILL_CATEGORIES.len() as u16 + 3) {
        let root = Line::from(vec![
            Span::styled(" ● ", th.accent()),
            Span::styled(format!("{} ({})", data::NAME, data::HANDLE), th.heading()),
        ]);
        frame.render_widget(Paragraph::new(root), Rect::new(inner.x, y, inner.width, 1));
        y += 2;
    }

    for (i, cat) in data::SKILL_CATEGORIES.iter().enumerate() {
        if y >= inner.y + inner.height {
            break;
        }
        let is_sel = i == sel;
        let style = if is_sel {
            th.selected_row()
        } else {
            Style::new().bg(th.panel)
        };
        let prefix = if is_sel { "▸ " } else { "  " };
        let label = format!("{} ({})", cat.name.get(lang), cat.skills.len());
        let line = Line::from(vec![
            Span::styled(prefix, if is_sel { th.accent() } else { th.dim() }),
            Span::styled(
                theme::fit(&label, inner.width.saturating_sub(2) as usize),
                if is_sel { th.heading() } else { th.text() },
            ),
        ]);
        let r = Rect::new(inner.x, y, inner.width, 1);
        frame.render_widget(Paragraph::new(line).style(style), r);
        hits.items.push((r, i));
        y += 1;
    }
}

fn render_skills(frame: &mut Frame, area: Rect, sel: usize, lang: Lang, th: &theme::Theme) {
    let cat = &data::SKILL_CATEGORIES[sel];
    let title = Line::from(vec![
        Span::styled(" ◆ ", th.sage_text()),
        Span::styled(format!("{} ", cat.name.get(lang)), th.heading()),
    ]);
    let block = th.panel_block(title, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let name_w = cat
        .skills
        .iter()
        .map(|s| theme::width(s.name))
        .max()
        .unwrap_or(10)
        .min(22);
    let bar_w = if w >= 52 { 10 } else { 6 };

    for sk in cat.skills {
        let mut spans = Vec::new();
        let pad = " ".repeat(name_w.saturating_sub(theme::width(sk.name)) + 2);
        spans.push(Span::styled(theme::fit(sk.name, name_w), th.heading()));
        spans.push(Span::raw(pad));
        spans.extend(theme::bar(
            sk.focus,
            10,
            bar_w,
            Style::new().fg(th.red),
            Style::new().fg(th.border),
        ));
        spans.push(Span::styled(format!(" {}/10", sk.focus), th.dim()));
        lines.push(Line::from(spans));
        lines.extend(theme::wrapped_lines(
            &format!("  {}", sk.note.get(lang)),
            w,
            th.text2(),
        ));
    }

    // Collect cross-links touching any skill in this category
    let mut rels: Vec<String> = Vec::new();
    for &(a, b) in &data::SKILL_LINKS {
        if cat.skills.iter().any(|s| s.name == a || s.name == b) {
            rels.push(format!("{a} ↔ {b}"));
        }
    }

    if !rels.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            if lang == Lang::En {
                "GRAPH CROSS-LINKS"
            } else {
                "BECERİ BAĞLANTILARI"
            },
            th.accent(),
        )));
        let refs: Vec<&str> = rels.iter().map(String::as_str).collect();
        lines.extend(theme::badge_lines(
            &refs,
            w,
            th.badge_soft(),
            Style::new().bg(th.panel),
        ));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
