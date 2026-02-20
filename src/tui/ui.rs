use crate::tui::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

fn build_highlight_spans<'a>(
    s: &'a str,
    normal: Style,
    highlight: Style,
    ranges: &[(usize, usize)],
) -> (Vec<Span<'a>>, usize) {
    let mut skipped = 0usize;
    let mut valid: Vec<(usize, usize)> = Vec::new();

    for (start, end) in ranges {
        if start >= end {
            skipped += 1;
            continue;
        }
        if *end > s.len() {
            skipped += 1;
            continue;
        }
        if !s.is_char_boundary(*start) || !s.is_char_boundary(*end) {
            skipped += 1;
            continue;
        }
        valid.push((*start, *end));
    }

    valid.sort_by_key(|(s, _e)| *s);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in valid {
        if let Some((_ps, pe)) = merged.last_mut() {
            if start <= *pe {
                *pe = (*pe).max(end);
                continue;
            }
        }
        merged.push((start, end));
    }

    let mut spans: Vec<Span<'a>> = Vec::new();
    let mut pos = 0usize;
    for (start, end) in merged {
        if pos < start {
            if let Some(seg) = s.get(pos..start) {
                spans.push(Span::styled(seg, normal));
            }
        }
        if let Some(seg) = s.get(start..end) {
            spans.push(Span::styled(seg, highlight));
            pos = end;
        } else {
            skipped += 1;
        }
    }
    if pos < s.len() {
        if let Some(seg) = s.get(pos..s.len()) {
            spans.push(Span::styled(seg, normal));
        }
    }

    (spans, skipped)
}

pub fn draw(frame: &mut Frame, app: &AppState) {
    let root = frame.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7)])
        .split(root);

    let main = rows[0];
    let status = rows[1];

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main);

    let left = cols[0];
    let right = cols[1];

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(right);

    render_lines_pane(frame, left, app);
    render_matches_pane(frame, right_rows[0], app);
    render_details_pane(frame, right_rows[1], app);
    render_status_pane(frame, status, app);
}

fn render_lines_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let title = format!("Lines (cursor: {})", app.cursor_line_idx + 1);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);

    let view_height = inner.height as usize;
    let total = app.lines.len();
    let cursor = app.cursor_line_idx.min(total.saturating_sub(1));

    let start = if view_height == 0 {
        0
    } else if cursor > view_height / 2 {
        cursor - (view_height / 2)
    } else {
        0
    };
    let mut end = (start + view_height).min(total);
    let mut start = start;
    if end.saturating_sub(start) < view_height {
        start = end.saturating_sub(view_height);
    }
    end = (start + view_height).min(total);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, line) in app.lines.iter().enumerate().take(end).skip(start) {
        let (has_matches, selected_match_ranges) = if let Some(report) = &app.last_good {
            let has = report
                .matches_by_line
                .get(idx)
                .is_some_and(|m| !m.is_empty());

            // For now, highlight capture spans for the first match on the selected line.
            let ranges = if idx == cursor {
                report
                    .matches_by_line
                    .get(idx)
                    .and_then(|m| m.first())
                    .map(|lm| {
                        lm.captures
                            .iter()
                            .map(|c| (c.start_byte, c.end_byte))
                            .collect::<Vec<(usize, usize)>>()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            (has, ranges)
        } else {
            (false, Vec::new())
        };

        let matched_line_bg = Style::default().bg(Color::Rgb(30, 30, 30));
        let selected_line_bg = Style::default().bg(Color::Rgb(0, 55, 95));
        let highlight_style = Style::default()
            .bg(Color::Rgb(110, 80, 0))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let mut base = Style::default();
        if has_matches {
            base = base.patch(matched_line_bg);
        }
        if idx == cursor {
            base = base
                .patch(selected_line_bg)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }

        let marker = if idx == cursor { ">" } else { " " };
        let marker_style = if idx == cursor {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .patch(selected_line_bg)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let num_style = base.fg(if idx == cursor {
            Color::White
        } else {
            Color::Gray
        });

        let prefix_num = format!("{:>5} ", idx + 1);

        let (text_spans, skipped) =
            build_highlight_spans(line.as_str(), base, highlight_style, &selected_match_ranges);

        let mut row: Vec<Span> = Vec::new();
        row.push(Span::styled(marker, marker_style));
        row.push(Span::styled(prefix_num, num_style));
        if skipped > 0 {
            // Non-fatal: bad offsets are skipped.
            row.push(Span::styled(
                "! ",
                Style::default().fg(Color::Yellow).patch(base),
            ));
        }
        row.extend(text_spans);
        lines.push(Line::from(row));
    }

    if lines.is_empty() {
        lines.push(Line::from("(no input loaded)"));
    }

    let text = Text::from(lines);
    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_matches_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default().borders(Borders::ALL).title("Matches");
    let cursor = app.cursor_line_idx;

    let mut lines: Vec<Line> = Vec::new();
    if let Some(report) = &app.last_good {
        if let Some(matches) = report.matches_by_line.get(cursor) {
            if matches.is_empty() {
                lines.push(Line::from("(no matches on this line)"));
            } else {
                for m in matches {
                    lines.push(Line::from(format!(
                        "#{} {} -> {} | line={} record={}",
                        m.rule_idx, m.state_before, m.state_after, m.line_action, m.record_action
                    )));
                }
            }
        } else {
            lines.push(Line::from("(cursor out of range)"));
        }
    } else {
        lines.push(Line::from("(no debug report loaded)"));
    }

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_details_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default().borders(Borders::ALL).title("Details");
    let cursor = app.cursor_line_idx;

    let mut lines: Vec<Line> = Vec::new();
    if let Some(report) = &app.last_good {
        let match_count = report
            .matches_by_line
            .get(cursor)
            .map(|m| m.len())
            .unwrap_or(0);
        let record_count = report
            .records
            .iter()
            .filter(|r| r.line_idx == cursor)
            .count();

        lines.push(Line::from(format!(
            "matches={} records_emitted_here={}",
            match_count, record_count
        )));

        if let Some(first_match) = report.matches_by_line.get(cursor).and_then(|m| m.first()) {
            lines.push(Line::from(""));
            lines.push(Line::from("captures:"));
            if first_match.captures.is_empty() {
                lines.push(Line::from("(no captures)"));
            } else {
                for c in &first_match.captures {
                    lines.push(Line::from(format!(
                        "- {} = {} (raw: {})",
                        c.name, c.typed, c.raw
                    )));
                }
            }
        }
    } else {
        lines.push(Line::from("No parse results yet."));
    }

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_status_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let status_str = match app.status {
        crate::tui::app::ParseStatus::Idle => "idle",
        crate::tui::app::ParseStatus::Parsing => "parsing",
        crate::tui::app::ParseStatus::Ok => "ok",
        crate::tui::app::ParseStatus::Error => "error",
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(status_str),
    ]));

    lines.push(Line::from(format!(
        "template: {}",
        app.template_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    )));
    lines.push(Line::from(format!(
        "input:    {}",
        app.input_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<missing>".to_string())
    )));

    if let Some(report) = &app.last_good {
        let cursor = app
            .cursor_line_idx
            .min(report.lines.len().saturating_sub(1));
        if let Some(first_match) = report.matches_by_line.get(cursor).and_then(|m| m.first()) {
            let line = report.lines.get(cursor).map(|s| s.as_str()).unwrap_or("");
            let mut bad = 0usize;
            for c in &first_match.captures {
                let ok = c.start_byte < c.end_byte
                    && c.end_byte <= line.len()
                    && line.is_char_boundary(c.start_byte)
                    && line.is_char_boundary(c.end_byte);
                if !ok {
                    bad += 1;
                }
            }
            if bad > 0 {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!(
                        "warning: {} capture span(s) had invalid byte offsets (skipped)",
                        bad
                    ),
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
    }

    if let Some(err) = &app.current_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "last parse error:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        for l in err.lines().take(3) {
            lines.push(Line::from(Span::styled(l, Style::default().fg(Color::Red))));
        }
        if err.lines().count() > 3 {
            lines.push(Line::from(Span::styled(
                "(truncated)",
                Style::default().fg(Color::Red),
            )));
        }
    }

    let title = if app.current_error.is_some() {
        "Status + Error"
    } else {
        "Status"
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}
