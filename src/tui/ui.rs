use crate::tui::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

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
        let has_matches = app
            .last_good
            .as_ref()
            .and_then(|r| r.matches_by_line.get(idx))
            .is_some_and(|m| !m.is_empty());

        let mut style = Style::default();
        if has_matches {
            style = style.fg(Color::Yellow);
        }
        if idx == cursor {
            style = style
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }

        let prefix = format!("{:>5} ", idx + 1);
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(line.as_str(), style),
        ]));
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
