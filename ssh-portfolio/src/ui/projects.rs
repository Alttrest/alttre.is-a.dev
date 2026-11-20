//! Projects tab: 13 real projects with category filter, split showcase, and expanded detail view.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};

use super::{App, is_activate, is_left, is_right, nav_of, step, theme};
use crate::data::{self, Lang, Project};
use crate::events::{KeyCode, KeyEvent};

fn visible_indices(app: &App) -> Vec<usize> {
    let filter = data::PROJECT_FILTERS[app.proj_filter % data::PROJECT_FILTERS.len()];
    data::filtered_projects(filter)
}

pub(crate) fn key(app: &mut App, k: &KeyEvent) -> bool {
    let list = visible_indices(app);
    let len = list.len();

    if app.proj_open {
        if k.code == KeyCode::Esc {
            app.proj_open = false;
            app.proj_scroll = 0;
            return true;
        }
        if is_activate(k) {
            app.proj_open = false;
            app.proj_scroll = 0;
            return true;
        }
        if is_left(k) {
            if len > 0 {
                app.proj_sel = app.proj_sel.saturating_sub(1);
                app.proj_scroll = 0;
            }
            return true;
        }
        if is_right(k) {
            if len > 0 {
                app.proj_sel = (app.proj_sel + 1).min(len - 1);
                app.proj_scroll = 0;
            }
            return true;
        }
        if let Some(nav) = nav_of(k) {
            match nav {
                super::Nav::Up => app.proj_scroll = app.proj_scroll.saturating_sub(1),
                super::Nav::Down => app.proj_scroll = app.proj_scroll.saturating_add(1),
                super::Nav::PageUp => app.proj_scroll = app.proj_scroll.saturating_sub(6),
                super::Nav::PageDown => app.proj_scroll = app.proj_scroll.saturating_add(6),
                super::Nav::Home => app.proj_scroll = 0,
                super::Nav::End => app.proj_scroll = app.proj_scroll.saturating_add(20),
            }
            return true;
        }
        return false;
    }

    if matches!(k.code, KeyCode::Char('f') | KeyCode::Char('F')) {
        app.proj_filter = (app.proj_filter + 1) % data::PROJECT_FILTERS.len();
        app.proj_sel = 0;
        app.proj_scroll = 0;
        app.proj_offset.set(0);
        return true;
    }
    if is_right(k) {
        app.proj_filter = (app.proj_filter + 1) % data::PROJECT_FILTERS.len();
        app.proj_sel = 0;
        app.proj_scroll = 0;
        app.proj_offset.set(0);
        return true;
    }
    if is_left(k) {
        app.proj_filter =
            (app.proj_filter + data::PROJECT_FILTERS.len() - 1) % data::PROJECT_FILTERS.len();
        app.proj_sel = 0;
        app.proj_scroll = 0;
        app.proj_offset.set(0);
        return true;
    }
    if is_activate(k) {
        if len > 0 {
            app.proj_open = true;
            app.proj_scroll = 0;
        }
        return true;
    }
    if let Some(nav) = nav_of(k) {
        app.proj_sel = step(app.proj_sel, len, nav, 5);
        app.proj_scroll = 0;
        return true;
    }
    false
}

pub(crate) fn click(app: &mut App, idx: usize) {
    let list = visible_indices(app);
    if idx < list.len() {
        if app.proj_sel == idx && !app.proj_open {
            app.proj_open = true;
        } else {
            app.proj_sel = idx;
            app.proj_scroll = 0;
        }
    }
}

pub(crate) fn scroll(app: &mut App, delta: i32) {
    let list = visible_indices(app);
    if app.proj_open {
        if delta < 0 {
            app.proj_scroll = app.proj_scroll.saturating_sub(2);
        } else {
            app.proj_scroll = app.proj_scroll.saturating_add(2);
        }
    } else if !list.is_empty() {
        let next = (app.proj_sel as isize + delta as isize).clamp(0, list.len() as isize - 1);
        app.proj_sel = next as usize;
    }
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let th = app.theme();
    let lang = app.lang;
    let list = visible_indices(app);
    let sel = app.proj_sel.min(list.len().saturating_sub(1));

    let [filter_bar, content] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas(area);

    render_filter_bar(frame, filter_bar, app, &th);

    if list.is_empty() {
        return;
    }
    let active_project = &data::PROJECTS[list[sel]];

    if app.proj_open {
        render_detail(frame, content, active_project, sel + 1, list.len(), app, &th, true);
        return;
    }

    if content.width < 72 {
        render_list(frame, content, &list, sel, app, &th);
    } else {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(42),
            Constraint::Percentage(58),
        ])
        .areas(content);
        render_list(frame, left, &list, sel, app, &th);
        render_detail(frame, right, active_project, sel + 1, list.len(), app, &th, false);
    }

    let _ = lang;
}

fn render_filter_bar(frame: &mut Frame, area: Rect, app: &App, th: &theme::Theme) {
    let lang = app.lang;
    let mut spans = vec![Span::raw(" ")];
    for (i, f) in data::PROJECT_FILTERS.iter().enumerate() {
        let count = data::filtered_projects(*f).len();
        let label = format!(" {} ({}) ", data::filter_label(*f, lang), count);
        let style = if i == app.proj_filter {
            th.badge_red()
        } else {
            th.badge_soft()
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).style(th.canvas()), area);
}

fn render_list(
    frame: &mut Frame,
    area: Rect,
    list: &[usize],
    sel: usize,
    app: &App,
    th: &theme::Theme,
) {
    let lang = app.lang;
    let title = Line::from(vec![
        Span::styled(
            if lang == Lang::En { " Projects " } else { " Projeler " },
            th.heading(),
        ),
        Span::styled(format!("{}/{} ", sel + 1, list.len()), th.dim()),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let rows_per_item: usize = if inner.height >= 12 { 2 } else { 1 };
    let visible_items = (inner.height as usize / rows_per_item).max(1);
    let offset = theme::scroll_window(app.proj_offset.get(), sel, visible_items, list.len());
    app.proj_offset.set(offset);

    let mut y = inner.y;
    let mut hits = app.hits.borrow_mut();
    for (vis_idx, &proj_idx) in list.iter().enumerate().skip(offset).take(visible_items) {
        if y >= inner.y + inner.height {
            break;
        }
        let p = &data::PROJECTS[proj_idx];
        let is_sel = vis_idx == sel;
        let row_style = if is_sel {
            th.selected_row()
        } else {
            Style::new().bg(th.panel)
        };
        let marker = if is_sel { "▸ " } else { "  " };
        let marker_style = if is_sel { th.accent() } else { th.dim() };
        let title_style = if is_sel { th.heading() } else { th.text() };

        let avail = inner.width as usize;
        let head_text = format!("{} {}", p.emoji, p.title);
        let max_title = avail.saturating_sub(4);
        let row1 = Line::from(vec![
            Span::styled(marker, marker_style),
            Span::styled(theme::fit(&head_text, max_title), title_style),
        ]);
        let r1 = Rect::new(inner.x, y, inner.width, 1);
        frame.render_widget(Paragraph::new(row1).style(row_style), r1);

        let mut hit_h = 1u16;
        if rows_per_item == 2 && y + 1 < inner.y + inner.height {
            let sub = format!("  {} · {}", p.category, p.tag);
            let row2 = Line::from(Span::styled(
                theme::fit(&sub, avail),
                if is_sel { th.sage_text() } else { th.dim() },
            ));
            let r2 = Rect::new(inner.x, y + 1, inner.width, 1);
            frame.render_widget(Paragraph::new(row2).style(row_style), r2);
            hit_h = 2;
        }
        hits.items.push((Rect::new(inner.x, y, inner.width, hit_h), vis_idx));
        y = y.saturating_add(hit_h);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_detail(
    frame: &mut Frame,
    area: Rect,
    p: &Project,
    pos: usize,
    total: usize,
    app: &App,
    th: &theme::Theme,
    expanded: bool,
) {
    let lang = app.lang;
    let title = Line::from(vec![
        Span::styled(format!(" {} {} ", p.emoji, p.title), th.heading()),
        Span::styled(format!("[{pos}/{total}] "), th.dim()),
    ]);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if expanded { th.red } else { th.border }))
        .style(Style::new().fg(th.ink).bg(th.panel))
        .padding(ratatui::widgets::Padding::horizontal(1))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let compact = inner.height < 18;
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(vec![
        theme::badge(p.category, th.badge_sage()),
        Span::raw(" "),
        theme::badge(p.tag, th.badge_red()),
    ]));
    if !compact {
        lines.push(Line::raw(""));
    }
    lines.extend(theme::wrapped_lines(p.short.get(lang), w, th.heading()));
    if !compact {
        lines.push(Line::raw(""));
        lines.extend(theme::wrapped_lines(p.full.get(lang), w, th.text2()));
        lines.push(Line::raw(""));
    } else {
        lines.extend(theme::wrapped_lines(p.full.get(lang), w, th.text2()));
    }

    lines.push(Line::from(Span::styled(
        if lang == Lang::En { "TECH STACK" } else { "TEKNOLOJİLER" },
        th.accent(),
    )));
    lines.extend(theme::badge_lines(
        p.stack,
        w,
        th.badge_soft(),
        Style::new().bg(th.panel),
    ));
    if !compact {
        lines.push(Line::raw(""));
    }

    lines.push(Line::from(Span::styled(
        if lang == Lang::En { "LINKS" } else { "BAĞLANTILAR" },
        th.accent(),
    )));
    lines.push(Line::from(vec![
        Span::styled("GitHub   ", th.dim()),
        Span::styled(p.github, th.rust_style()),
    ]));
    if let Some(live) = p.live {
        lines.push(Line::from(vec![
            Span::styled("Live     ", th.dim()),
            Span::styled(live, th.sage_text()),
        ]));
    }
    if let Some(devpost) = p.devpost {
        lines.push(Line::from(vec![
            Span::styled("Devpost  ", th.dim()),
            Span::styled(devpost, th.text2()),
        ]));
    }

    if expanded {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            if lang == Lang::En {
                "Esc to return · ←/→ previous/next project"
            } else {
                "Geri dönmek için Esc · ←/→ önceki/sonraki proje"
            },
            th.dim(),
        )));
    }

    let max_scroll = lines.len().saturating_sub(inner.height as usize) as u16;
    let scroll = app.proj_scroll.min(max_scroll);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        inner,
    );
}
