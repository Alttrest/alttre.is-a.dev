//! Competitions ("Yarışmalar") tab: SAHADAN NOTLAR lanyard badges with flip mechanic.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Wrap};

use super::{App, is_activate, is_left, is_right, nav_of, step, theme};
use crate::data::{self, Competition, Lang};
use crate::events::KeyEvent;

pub(crate) fn key(app: &mut App, k: &KeyEvent) -> bool {
    let len = data::COMPETITIONS.len();
    if is_left(k) {
        app.comp_sel = (app.comp_sel + len - 1) % len;
        return true;
    }
    if is_right(k) {
        app.comp_sel = (app.comp_sel + 1) % len;
        return true;
    }
    if is_activate(k) {
        let i = app.comp_sel.min(app.comp_flipped.len() - 1);
        app.comp_flipped[i] = !app.comp_flipped[i];
        return true;
    }
    if let Some(nav) = nav_of(k) {
        app.comp_sel = step(app.comp_sel, len, nav, 1);
        return true;
    }
    false
}

pub(crate) fn click(app: &mut App, idx: usize) {
    if idx < data::COMPETITIONS.len() {
        if app.comp_sel == idx {
            app.comp_flipped[idx] = !app.comp_flipped[idx];
        } else {
            app.comp_sel = idx;
        }
    }
}

pub(crate) fn scroll(app: &mut App, delta: i32) {
    let len = data::COMPETITIONS.len();
    let next = (app.comp_sel as isize + delta as isize).clamp(0, len as isize - 1);
    app.comp_sel = next as usize;
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let th = app.theme();
    let lang = app.lang;

    let banner_h = if area.height >= 16 { 4 } else { 2 };
    let [head, cards_area] = Layout::vertical([
        Constraint::Length(banner_h),
        Constraint::Min(1),
    ])
    .areas(area);

    render_banner(frame, head, lang, &th);

    let sel = app.comp_sel.min(data::COMPETITIONS.len() - 1);
    if cards_area.width >= 78 {
        let cols = Layout::horizontal([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .areas::<3>(cards_area);
        let mut hits = app.hits.borrow_mut();
        for (i, &rect) in cols.iter().enumerate() {
            render_card(frame, rect, &data::COMPETITIONS[i], i == sel, app.comp_flipped[i], lang, &th);
            hits.items.push((rect, i));
        }
    } else {
        // Compact single-card view with selector strip
        let [strip, card_rect] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .areas(cards_area);
        let mut spans = vec![Span::raw(" ")];
        for (i, c) in data::COMPETITIONS.iter().enumerate() {
            let st = if i == sel { th.badge_red() } else { th.badge_soft() };
            spans.push(Span::styled(format!(" {} {} ", c.glyph, c.year), st));
            spans.push(Span::raw(" "));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)).style(th.canvas()), strip);
        render_card(
            frame,
            card_rect,
            &data::COMPETITIONS[sel],
            true,
            app.comp_flipped[sel],
            lang,
            &th,
        );
        app.hits.borrow_mut().items.push((card_rect, sel));
    }
}

fn render_banner(frame: &mut Frame, area: Rect, lang: Lang, th: &theme::Theme) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(data::COMP_KICKER, th.accent()),
        Span::styled("  ·  ", th.dim()),
        Span::styled(data::COMP_TITLE.get(lang), th.heading()),
        Span::styled("  ", th.dim()),
        Span::styled(data::COMP_TAGLINE.get(lang), th.text2()),
    ]));

    if area.height >= 3 {
        let stats = if lang == Lang::En {
            format!(
                "{} Hackathons · {} Award 🏆",
                data::HACKATHON_COUNT,
                data::AWARD_COUNT
            )
        } else {
            format!(
                "{} Hackathon · {} Ödül 🏆",
                data::HACKATHON_COUNT,
                data::AWARD_COUNT
            )
        };
        let mut spans = vec![Span::raw(" "), theme::badge(stats, th.badge_red()), Span::raw(" ")];
        for item in &data::COMP_BANNER {
            spans.push(theme::badge(item.get(lang), th.badge_soft()));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::raw(""));
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines).style(th.canvas()), area);
}

#[allow(clippy::too_many_arguments)]
fn render_card(
    frame: &mut Frame,
    area: Rect,
    c: &Competition,
    focused: bool,
    flipped: bool,
    lang: Lang,
    th: &theme::Theme,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let tone = Color::Rgb(c.tone.0, c.tone.1, c.tone.2);
    let border_color = if focused { th.red } else { tone };
    let side_tag = if flipped {
        if lang == Lang::En { "BACK" } else { "ARKA" }
    } else if lang == Lang::En {
        "FRONT"
    } else {
        "ÖN"
    };
    let title = Line::from(vec![
        Span::styled(format!(" {} ", c.glyph), Style::new().fg(tone).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{} ", c.number), th.dim()),
        Span::styled(format!("[{side_tag}] "), if focused { th.accent() } else { th.dim() }),
    ]);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border_color))
        .style(Style::new().fg(th.ink).bg(th.panel))
        .padding(Padding::horizontal(1))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let w = inner.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    if !flipped {
        // Lanyard strap motif
        lines.push(Line::from(Span::styled(
            format!("─── {} · {} ───", c.year, c.kind.get(lang)),
            Style::new().fg(tone).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::raw(""));
        lines.extend(theme::wrapped_lines(c.name, w, th.heading()));
        if let Some(award) = &c.award {
            lines.push(Line::raw(""));
            lines.push(Line::from(theme::badge(
                format!("🏆 {}", award.get(lang)),
                th.badge_red(),
            )));
        }
        lines.push(Line::raw(""));
        lines.extend(theme::wrapped_lines(c.blurb.get(lang), w, th.text2()));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("ID  ", th.dim()),
            Span::styled(c.id_code, Style::new().fg(tone)),
        ]));
        lines.push(Line::from(Span::styled("▌█▐▌█▌▐█▐▌█▌", th.dim())));
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            if lang == Lang::En {
                "[Enter / Space] Flip badge ↻"
            } else {
                "[Enter / Space] Kartı çevir ↻"
            },
            if focused { th.accent() } else { th.dim() },
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("{} // {}", c.name, c.id_code),
            Style::new().fg(tone).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::raw(""));
        for item in c.back {
            lines.extend(theme::wrapped_lines(item.get(lang), w, th.text()));
        }
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            if lang == Lang::En {
                "[Enter / Space] Back to front ↺"
            } else {
                "[Enter / Space] Ön yüze dön ↺"
            },
            if focused { th.accent() } else { th.dim() },
        )));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
