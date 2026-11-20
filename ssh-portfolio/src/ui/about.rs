//! About & Dev Logs tab: Ali Turan bio, Nebula Team, philosophy, and 5 readable engineering logs.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::{App, is_activate, nav_of, step, theme};
use crate::data::{self, Lang};
use crate::events::{KeyCode, KeyEvent};

pub(crate) fn key(app: &mut App, k: &KeyEvent) -> bool {
    let len = data::DEV_LOGS.len();
    if app.about_open {
        if k.code == KeyCode::Esc {
            app.about_open = false;
            app.about_scroll = 0;
            return true;
        }
        if matches!(k.code, KeyCode::Char('n') | KeyCode::Char('N')) {
            app.about_sel = (app.about_sel + 1).min(len - 1);
            app.about_scroll = 0;
            return true;
        }
        if matches!(k.code, KeyCode::Char('p') | KeyCode::Char('P')) {
            app.about_sel = app.about_sel.saturating_sub(1);
            app.about_scroll = 0;
            return true;
        }
        if let Some(nav) = nav_of(k) {
            match nav {
                super::Nav::Up => app.about_scroll = app.about_scroll.saturating_sub(1),
                super::Nav::Down => app.about_scroll = app.about_scroll.saturating_add(1),
                super::Nav::PageUp => app.about_scroll = app.about_scroll.saturating_sub(6),
                super::Nav::PageDown => app.about_scroll = app.about_scroll.saturating_add(6),
                super::Nav::Home => app.about_scroll = 0,
                super::Nav::End => app.about_scroll = app.about_scroll.saturating_add(30),
            }
            return true;
        }
        return false;
    }

    if is_activate(k) {
        app.about_open = true;
        app.about_scroll = 0;
        return true;
    }
    if matches!(k.code, KeyCode::Char('n') | KeyCode::Char('N')) {
        app.about_sel = (app.about_sel + 1).min(len - 1);
        return true;
    }
    if matches!(k.code, KeyCode::Char('p') | KeyCode::Char('P')) {
        app.about_sel = app.about_sel.saturating_sub(1);
        return true;
    }
    if let Some(nav) = nav_of(k) {
        app.about_sel = step(app.about_sel, len, nav, 2);
        return true;
    }
    false
}

pub(crate) fn click(app: &mut App, idx: usize) {
    if idx < data::DEV_LOGS.len() {
        if app.about_sel == idx && !app.about_open {
            app.about_open = true;
            app.about_scroll = 0;
        } else {
            app.about_sel = idx;
        }
    }
}

pub(crate) fn scroll(app: &mut App, delta: i32) {
    if app.about_open {
        if delta < 0 {
            app.about_scroll = app.about_scroll.saturating_sub(2);
        } else {
            app.about_scroll = app.about_scroll.saturating_add(2);
        }
    } else {
        let len = data::DEV_LOGS.len();
        let next = (app.about_sel as isize + delta as isize).clamp(0, len as isize - 1);
        app.about_sel = next as usize;
    }
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let th = app.theme();

    if app.about_open {
        render_log_reader(frame, area, app, &th);
        return;
    }

    if area.width >= 76 {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(52),
            Constraint::Percentage(48),
        ])
        .areas(area);
        render_profile(frame, left, app, &th);
        render_logs_list(frame, right, app, &th);
    } else {
        let [top, bottom] = Layout::vertical([
            Constraint::Percentage(52),
            Constraint::Percentage(48),
        ])
        .areas(area);
        render_profile(frame, top, app, &th);
        render_logs_list(frame, bottom, app, &th);
    }
}

fn render_profile(frame: &mut Frame, area: Rect, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let title = Line::from(vec![
        Span::styled(" A_ ", th.badge_red()),
        Span::styled(
            if lang == Lang::En {
                " About Ali Turan "
            } else {
                " Ali Turan Hakkında "
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
        Span::styled(data::NAME, th.heading()),
        Span::styled(format!(" ({})  ·  ", data::HANDLE), th.accent()),
        Span::styled(data::SITE, th.dim()),
    ]));
    lines.push(Line::raw(""));
    lines.extend(theme::wrapped_lines(data::ABOUT_BIO.get(lang), w, th.text2()));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("NEBULA TEAM", th.accent())));
    lines.extend(theme::wrapped_lines(data::NEBULA.get(lang), w, th.text()));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        if lang == Lang::En {
            "DEVELOPER PHILOSOPHY"
        } else {
            "GELİŞTİRİCİ FELSEFESİ"
        },
        th.accent(),
    )));
    for (head, body) in &data::PHILOSOPHY {
        lines.push(Line::from(vec![
            Span::styled("▸ ", th.accent()),
            Span::styled(head.get(lang), th.heading()),
        ]));
        lines.extend(theme::wrapped_lines(body.get(lang), w, th.text2()));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_logs_list(frame: &mut Frame, area: Rect, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let sel = app.about_sel.min(data::DEV_LOGS.len() - 1);
    let title = Line::from(vec![
        Span::styled(
            if lang == Lang::En {
                " Dev Logs "
            } else {
                " Geliştirici Notları "
            },
            th.heading(),
        ),
        Span::styled(format!("({}/{}) ", sel + 1, data::DEV_LOGS.len()), th.dim()),
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

    for (i, log) in data::DEV_LOGS.iter().enumerate() {
        if y >= inner.y + inner.height {
            break;
        }
        let is_sel = i == sel;
        let row_style = if is_sel {
            th.selected_row()
        } else {
            Style::new().bg(th.panel)
        };
        let r1 = Rect::new(inner.x, y, inner.width, 1);
        let line1 = Line::from(vec![
            Span::styled(if is_sel { "▸ " } else { "  " }, if is_sel { th.accent() } else { th.dim() }),
            Span::styled(
                theme::fit(log.title.get(lang), w.saturating_sub(2)),
                if is_sel { th.heading() } else { th.text() },
            ),
        ]);
        frame.render_widget(Paragraph::new(line1).style(row_style), r1);

        let mut h = 1u16;
        if y + 1 < inner.y + inner.height {
            let meta = format!("  {} · {} · {} min", log.category.get(lang), log.date, log.read_min);
            let r2 = Rect::new(inner.x, y + 1, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    theme::fit(&meta, w),
                    if is_sel { th.sage_text() } else { th.dim() },
                )))
                .style(row_style),
                r2,
            );
            h = 2;
        }
        if is_sel && y + 2 < inner.y + inner.height && inner.height >= 14 {
            let r3 = Rect::new(inner.x, y + 2, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    theme::fit(&format!("  {}", log.excerpt.get(lang)), w),
                    th.text2(),
                )))
                .style(row_style),
                r3,
            );
            h = 3;
        }
        hits.items.push((Rect::new(inner.x, y, inner.width, h), i));
        y = y.saturating_add(h);
    }
}

fn render_log_reader(frame: &mut Frame, area: Rect, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let sel = app.about_sel.min(data::DEV_LOGS.len() - 1);
    let log = &data::DEV_LOGS[sel];

    let title = Line::from(vec![
        Span::styled(format!(" [{}/{}] ", sel + 1, data::DEV_LOGS.len()), th.badge_red()),
        Span::styled(format!(" {} ", log.title.get(lang)), th.heading()),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(vec![
        theme::badge(log.category.get(lang), th.badge_sage()),
        Span::styled(format!("  {}  ·  {} min", log.date, log.read_min), th.dim()),
    ]));
    lines.push(Line::raw(""));
    lines.extend(theme::badge_lines(
        log.tags,
        w,
        th.badge_soft(),
        Style::new().bg(th.panel),
    ));
    lines.push(Line::raw(""));
    lines.extend(theme::wrapped_lines(log.excerpt.get(lang), w, th.text2()));

    for (heading, body) in log.sections {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(heading.get(lang), th.accent())));
        lines.extend(theme::wrapped_lines(body.get(lang), w, th.text()));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("─".repeat(w.min(40)), th.dim())));
    lines.extend(theme::wrapped_lines(log.callout.get(lang), w, th.rust_style()));

    let max_scroll = lines.len().saturating_sub(inner.height as usize) as u16;
    let scroll = app.about_scroll.min(max_scroll);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        inner,
    );
}
