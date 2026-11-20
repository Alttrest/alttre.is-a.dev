//! Contact tab: social/contact cards, quick message prompt, and Istanbul availability note.

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
        app.contact_sel = step(app.contact_sel, data::CONTACTS.len(), nav, 3);
        return true;
    }
    false
}

pub(crate) fn click(app: &mut App, idx: usize) {
    if idx < data::CONTACTS.len() {
        app.contact_sel = idx;
    }
}

pub(crate) fn scroll(app: &mut App, delta: i32) {
    let len = data::CONTACTS.len();
    let next = (app.contact_sel as isize + delta as isize).clamp(0, len as isize - 1);
    app.contact_sel = next as usize;
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let th = app.theme();
    let sel = app.contact_sel.min(data::CONTACTS.len() - 1);

    if area.width >= 76 {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .areas(area);
        render_channels(frame, left, sel, app, &th);
        render_card(frame, right, sel, app, &th);
    } else {
        let [top, bottom] = Layout::vertical([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .areas(area);
        render_channels(frame, top, sel, app, &th);
        render_card(frame, bottom, sel, app, &th);
    }
}

fn render_channels(frame: &mut Frame, area: Rect, sel: usize, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let title = Line::from(vec![
        Span::styled(
            if lang == Lang::En {
                " Channels & Links "
            } else {
                " İletişim & Bağlantılar "
            },
            th.heading(),
        ),
        Span::styled(format!("({}) ", data::CONTACTS.len()), th.dim()),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let mut y = inner.y;
    let mut hits = app.hits.borrow_mut();

    for (i, c) in data::CONTACTS.iter().enumerate() {
        if y >= inner.y + inner.height {
            break;
        }
        let is_sel = i == sel;
        let row_style = if is_sel {
            th.selected_row()
        } else {
            Style::new().bg(th.panel)
        };
        let label = c.label.get(lang);
        let head = format!("{} {}  {}", c.icon, label, c.value);
        let line1 = Line::from(vec![
            Span::styled(if is_sel { "▸ " } else { "  " }, if is_sel { th.accent() } else { th.dim() }),
            Span::styled(
                theme::fit(&head, w.saturating_sub(2)),
                if is_sel { th.heading() } else { th.text() },
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line1).style(row_style),
            Rect::new(inner.x, y, inner.width, 1),
        );

        let mut h = 1u16;
        if y + 1 < inner.y + inner.height && inner.height >= 14 {
            let note = format!("    {}", c.note.get(lang));
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    theme::fit(&note, w),
                    if is_sel { th.sage_text() } else { th.dim() },
                )))
                .style(row_style),
                Rect::new(inner.x, y + 1, inner.width, 1),
            );
            h = 2;
        }
        hits.items.push((Rect::new(inner.x, y, inner.width, h), i));
        y = y.saturating_add(h);
    }
}

fn render_card(frame: &mut Frame, area: Rect, sel: usize, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let c = &data::CONTACTS[sel];
    let title = Line::from(vec![
        Span::styled(" ✉ ", th.badge_red()),
        Span::styled(
            if lang == Lang::En {
                " Say Hello "
            } else {
                " Merhaba De "
            },
            th.heading(),
        ),
    ]);
    let block = th.panel_block(title, false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("{} {}", c.icon, c.label.get(lang)), th.heading()),
    ]));
    lines.push(Line::from(Span::styled(c.value, th.accent())));
    lines.push(Line::from(Span::styled(c.url, th.rust_style())));
    lines.push(Line::raw(""));
    lines.extend(theme::wrapped_lines(c.note.get(lang), w, th.text2()));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        if lang == Lang::En {
            "MESSAGE TEMPLATE"
        } else {
            "MESAJ ŞABLONU"
        },
        th.accent(),
    )));
    lines.extend(theme::wrapped_lines(data::CONTACT_PROMPT.get(lang), w, th.text()));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("─".repeat(w.min(36)), th.dim())));
    lines.extend(theme::wrapped_lines(data::CONTACT_NOTE.get(lang), w, th.sage_text()));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
