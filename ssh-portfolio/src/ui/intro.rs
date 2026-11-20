//! Tab 1 — Intro: stylized `A_` / Alttre chevron banner, ASCII robot mascot,
//! live typewriter, summary blurb, quick stats bar, and `[Enter]` CTA.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{App, Tab, is_activate, theme};
use crate::data::{self, Lang};
use crate::events::KeyEvent;

pub(crate) fn key(app: &mut App, k: &KeyEvent) -> bool {
    if is_activate(k) {
        app.goto(Tab::Projects);
        true
    } else {
        false
    }
}

pub(crate) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let th = app.theme();
    let lang = app.lang;
    let tr = lang == Lang::Tr;

    let title = Line::from(vec![
        Span::styled(" 01 ", th.badge_red()),
        Span::styled(
            if tr { " GİRİŞ · ALTTRE // 2026 " } else { " INTRO · ALTTRE // 2026 " },
            th.heading(),
        ),
    ]);
    let block = th.panel_block(title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let wide = inner.width >= 68 && inner.height >= 10;
    let (main_area, mascot_area) = if wide {
        let [l, r] = Layout::horizontal([Constraint::Min(36), Constraint::Length(26)]).areas(inner);
        (l, Some(r))
    } else {
        (inner, None)
    };

    let w = main_area.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    // Stylized A_ / chevron banner (`ALTTRE // 2026`).
    if main_area.height >= 14 && w >= 38 {
        lines.push(Line::from(vec![
            Span::styled("   /\\   ", th.accent()),
            Span::styled(data::STAMP, th.heading()),
            Span::styled("  ·  ", th.dim()),
            Span::styled(data::SITE, th.rust_style()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  /", th.accent()),
            Span::styled("██", Style::new().fg(th.red)),
            Span::styled("\\  ", th.accent()),
            Span::styled(data::NAME, th.heading()),
            Span::styled(format!(" ({}) ", data::HANDLE), th.text2()),
            Span::styled(format!("· GitHub {}", data::GITHUB_HANDLE), th.dim()),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" /_/\\_\\ ", th.accent()),
            Span::styled("██ ", Style::new().fg(th.red)),
            Span::styled(data::ROLE.get(lang), th.sage_text()),
        ]));
        lines.push(Line::raw(""));
    } else {
        lines.push(Line::from(vec![
            Span::styled(" A_ ", th.badge_red()),
            Span::styled(format!(" {} ({}) · {} ", data::NAME, data::HANDLE, data::STAMP), th.heading()),
        ]));
    }

    // Live typewriter line + blinking cursor.
    let typed = app.typed_text();
    let cursor = if (app.tick / 3).is_multiple_of(2) { "█" } else { " " };
    let tw_prefix = "❯ ";
    let tw_body = theme::fit(&typed, w.saturating_sub(4));
    lines.push(Line::from(vec![
        Span::styled(tw_prefix, th.accent()),
        Span::styled(tw_body, th.heading()),
        Span::styled(cursor, Style::new().fg(th.red)),
    ]));

    // Progress dots across the 5 intro lines.
    let mut dots: Vec<Span> = vec![Span::styled("  ", th.dim())];
    for i in 0..data::INTRO_LINES.len() {
        let sym = if i == app.type_line { "● " } else { "◦ " };
        let st = if i == app.type_line { th.accent() } else { th.dim() };
        dots.push(Span::styled(sym, st));
    }
    lines.push(Line::from(dots));
    lines.push(Line::raw(""));

    // Summary blurb + Nebula note.
    if main_area.height >= 9 {
        lines.extend(theme::wrapped_lines(data::INTRO_SUMMARY.get(lang), w, th.text2()));
        lines.push(Line::raw(""));
    }

    // Quick stats bar: 13 Projects · 22 Skills · 4 Hackathons · 1 Award 🏆
    let stats_line = if tr {
        format!(
            "{} Proje · {} Yetenek · {} Hackathon · {} Ödül 🏆",
            data::PROJECTS.len(),
            data::skill_count(),
            data::HACKATHON_COUNT,
            data::AWARD_COUNT
        )
    } else {
        format!(
            "{} Projects · {} Skills · {} Hackathons · {} Award 🏆",
            data::PROJECTS.len(),
            data::skill_count(),
            data::HACKATHON_COUNT,
            data::AWARD_COUNT
        )
    };
    lines.push(Line::from(vec![
        theme::badge("STATS", th.badge_sage()),
        Span::raw(" "),
        Span::styled(stats_line, th.heading()),
    ]));
    lines.push(Line::raw(""));

    // CTA button + quick section jump hints.
    let cta = if tr {
        " [Enter] Projeleri Keşfet → "
    } else {
        " [Enter] Explore Projects → "
    };
    let jump = if tr {
        "  [1-6] bölümler · [e] TR/EN · [t] tema · [?] yardım"
    } else {
        "  [1-6] sections · [e] EN/TR · [t] theme · [?] help"
    };
    lines.push(Line::from(vec![
        Span::styled(cta, th.badge_red()),
        Span::styled(theme::fit(jump, w.saturating_sub(theme::width(cta))), th.dim()),
    ]));

    // Compact mascot inline when narrow but tall enough.
    if mascot_area.is_none() && main_area.height as usize > lines.len() + data::MASCOT.len() {
        lines.push(Line::raw(""));
        for row in data::MASCOT {
            lines.push(Line::from(Span::styled(row, th.rust_style())));
        }
    }

    frame.render_widget(Paragraph::new(lines), main_area);

    if let Some(m_area) = mascot_area {
        render_mascot(frame, m_area, app, &th);
    }
}

fn render_mascot(frame: &mut Frame, area: Rect, app: &App, th: &theme::Theme) {
    let tr = app.lang == Lang::Tr;
    let blink = (app.tick / 14) % 7 == 6;
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        if tr { "NEBULA · BİRİM 01" } else { "NEBULA · UNIT 01" },
        th.dim(),
    )));
    lines.push(Line::raw(""));
    for (i, row) in data::MASCOT.iter().enumerate() {
        let text = if i == 2 && blink {
            "     |  /  ─ ─  \\  |"
        } else {
            *row
        };
        let style = if i == 2 {
            Style::new().fg(th.red).add_modifier(Modifier::BOLD)
        } else {
            th.rust_style()
        };
        lines.push(Line::from(Span::styled(text, style)));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("● ", Style::new().fg(th.sage)),
        Span::styled(if tr { "SSH OTURUMU AKTİF" } else { "SSH SESSION LIVE" }, th.sage_text()),
    ]));
    lines.push(Line::from(Span::styled("ThinkPad TrackPoint UI", th.dim())));

    let h = (lines.len() as u16).min(area.height);
    let centered = theme::center(area, area.width, h);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), centered);
}
