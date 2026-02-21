use crate::tui::app::{AppState, Mode, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::collections::HashMap;

fn display_value_compact(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn record_preview(record: &HashMap<String, serde_json::Value>, max_len: usize) -> String {
    if record.is_empty() {
        return "(empty)".to_string();
    }

    let mut keys: Vec<&String> = record.keys().collect();
    keys.sort();

    let mut out = String::new();
    for (i, k) in keys.iter().enumerate() {
        let v = record.get(*k).unwrap();
        let part = format!("{}={}", k, display_value_compact(v));
        if i > 0 {
            if out.len() + 1 > max_len {
                break;
            }
            out.push(' ');
        }
        if out.len() + part.len() > max_len {
            break;
        }
        out.push_str(&part);
    }

    if out.is_empty() {
        "...".to_string()
    } else {
        out
    }
}

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
    if app.mode == Mode::Picker {
        draw_picker(frame, app);
        return;
    }

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

fn draw_picker(frame: &mut Frame, app: &AppState) {
    let root = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7)])
        .split(root);

    let main = rows[0];
    let status = rows[1];

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main);

    render_picker_list(frame, cols[0], app);
    render_picker_details(frame, cols[1], app);
    render_picker_status(frame, status, app);
}

fn render_picker_list(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(picker) = &app.picker else {
        let block = Block::default().borders(Borders::ALL).title("Picker");
        frame.render_widget(
            Paragraph::new("(picker not initialized)").block(block),
            area,
        );
        return;
    };

    let target = match picker.target {
        crate::tui::picker::PickTarget::Template => "Template",
        crate::tui::picker::PickTarget::Input => "Input",
    };
    let title = format!(
        "Pick {} (Enter open/select, Backspace up, i manual, Tab switch)",
        target
    );
    let block = Block::default().borders(Borders::ALL).title(title);

    let mut lines: Vec<Line> = Vec::new();
    if picker.entries.is_empty() {
        lines.push(Line::from("(empty directory)"));
    } else {
        for (i, ent) in picker.entries.iter().enumerate() {
            let is_sel = i == picker.selected_idx;
            let prefix = if is_sel { "> " } else { "  " };
            let mut name = ent.name.clone();
            if ent.is_dir {
                name.push('/');
            }
            let mut style = Style::default();
            if is_sel {
                style = style
                    .bg(Color::Rgb(50, 50, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            } else if ent.is_dir {
                style = style.fg(Color::Cyan);
            }
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, name),
                style,
            )));
        }
    }

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_picker_details(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(picker) = &app.picker else {
        let block = Block::default().borders(Borders::ALL).title("Details");
        frame.render_widget(Paragraph::new("(no picker)").block(block), area);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "cliscrape debug: pick template + input",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(format!("cwd: {}", picker.cwd.display())));
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

    lines.push(Line::from(""));
    if picker.manual {
        lines.push(Line::from(Span::styled(
            "manual path entry (Enter accept, Esc cancel):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::raw(format!("> {}", picker.manual_input))));
    } else {
        lines.push(Line::from("press `i` to type a path manually"));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("keys:"));
    lines.push(Line::from("- up/down or j/k: move"));
    lines.push(Line::from("- enter: open dir / select file"));
    lines.push(Line::from("- backspace: up directory"));
    lines.push(Line::from("- tab: switch template/input"));
    lines.push(Line::from("- q: quit"));

    if let Some(err) = &picker.last_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "picker error:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        for l in err.lines().take(3) {
            lines.push(Line::from(Span::styled(l, Style::default().fg(Color::Red))));
        }
    }

    let block = Block::default().borders(Borders::ALL).title("Details");
    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_picker_status(frame: &mut Frame, area: Rect, app: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "mode: picker",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if let Some(err) = &app.current_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "error:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        for l in err.lines().take(2) {
            lines.push(Line::from(Span::styled(l, Style::default().fg(Color::Red))));
        }
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from("select paths to start parsing"));
    }

    let block = Block::default().borders(Borders::ALL).title("Status");
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
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

            // Highlight capture spans for the selected match on the cursor line.
            let ranges = if idx == cursor && app.view_mode == ViewMode::Matches {
                report
                    .matches_by_line
                    .get(idx)
                    .and_then(|m| m.get(app.selected_match_idx))
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
    let title = match app.view_mode {
        ViewMode::Matches => "Matches",
        ViewMode::Records => "Records",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let cursor = app.cursor_line_idx;

    let mut lines: Vec<Line> = Vec::new();
    if let Some(report) = &app.last_good {
        match app.view_mode {
            ViewMode::Matches => {
                let selected = app.selected_match_idx;
                if let Some(matches) = report.matches_by_line.get(cursor) {
                    if matches.is_empty() {
                        lines.push(Line::from("(no matches on this line)"));
                    } else {
                        for (i, m) in matches.iter().enumerate() {
                            let is_sel = i == selected;
                            let prefix = if is_sel { ">" } else { " " };
                            let mut style = Style::default();
                            if is_sel {
                                style = style
                                    .bg(Color::Rgb(50, 50, 50))
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD);
                            }

                            let next = m
                                .next_state
                                .as_ref()
                                .map(|s| format!(" next={}", s))
                                .unwrap_or_default();

                            lines.push(Line::from(Span::styled(
                                format!(
                                    "{} [{}] r#{} | {} / {} | {} -> {}{}",
                                    prefix,
                                    i + 1,
                                    m.rule_idx,
                                    m.line_action,
                                    m.record_action,
                                    m.state_before,
                                    m.state_after,
                                    next
                                ),
                                style,
                            )));
                        }
                    }
                } else {
                    lines.push(Line::from("(cursor out of range)"));
                }
            }
            ViewMode::Records => {
                let selected = app.selected_record_idx;
                if report.records.is_empty() {
                    lines.push(Line::from("(no records emitted)"));
                } else {
                    for (i, rec) in report.records.iter().enumerate() {
                        let is_sel = i == selected;
                        let prefix = if is_sel { ">" } else { " " };
                        let mut style = Style::default();
                        if is_sel {
                            style = style
                                .bg(Color::Rgb(50, 50, 50))
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD);
                        }
                        let preview = record_preview(&rec.record, 70);
                        lines.push(Line::from(Span::styled(
                            format!(
                                "{} [{}] line={} {}",
                                prefix,
                                i + 1,
                                rec.line_idx + 1,
                                preview
                            ),
                            style,
                        )));
                    }
                }
            }
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
    if app.mode == Mode::EditTemplate {
        render_editor_pane(frame, area, app);
        return;
    }

    let title = match app.view_mode {
        ViewMode::Matches => "Details",
        ViewMode::Records => "Record Details",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let cursor = app.cursor_line_idx;

    let mut lines: Vec<Line> = Vec::new();
    if let Some(report) = &app.last_good {
        match app.view_mode {
            ViewMode::Matches => {
                let selected = app.selected_match_idx;
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

                if let Some(sel_match) = report
                    .matches_by_line
                    .get(cursor)
                    .and_then(|m| m.get(selected))
                {
                    lines.push(Line::from(""));
                    lines.push(Line::from(format!(
                        "match {}/{}",
                        selected + 1,
                        match_count.max(1)
                    )));
                    lines.push(Line::from(format!(
                        "rule: #{} | {} -> {} | line={} record={}",
                        sel_match.rule_idx,
                        sel_match.state_before,
                        sel_match.state_after,
                        sel_match.line_action,
                        sel_match.record_action
                    )));
                    if let Some(ns) = &sel_match.next_state {
                        lines.push(Line::from(format!("next_state: {}", ns)));
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from("captures:"));
                    if sel_match.captures.is_empty() {
                        lines.push(Line::from("(no captures)"));
                    } else {
                        for c in &sel_match.captures {
                            lines.push(Line::from(format!(
                                "- {} = {} (raw: {})",
                                c.name,
                                display_value_compact(&c.typed),
                                c.raw
                            )));
                        }
                    }
                } else {
                    lines.push(Line::from(""));
                    lines.push(Line::from("(no selected match)"));
                }
            }
            ViewMode::Records => {
                let selected = app.selected_record_idx;
                lines.push(Line::from(format!(
                    "records_total={}",
                    report.records.len()
                )));

                if let Some(rec) = report.records.get(selected) {
                    lines.push(Line::from(""));
                    lines.push(Line::from(format!(
                        "record {}/{} (line {})",
                        selected + 1,
                        report.records.len().max(1),
                        rec.line_idx + 1
                    )));
                    lines.push(Line::from(""));
                    lines.push(Line::from("fields:"));

                    let mut keys: Vec<&String> = rec.record.keys().collect();
                    keys.sort();
                    if keys.is_empty() {
                        lines.push(Line::from("(empty record)"));
                    } else {
                        for k in keys {
                            let v = rec.record.get(k).unwrap();
                            lines.push(Line::from(format!(
                                "- {} = {}",
                                k,
                                display_value_compact(v)
                            )));
                        }
                    }
                } else {
                    lines.push(Line::from(""));
                    lines.push(Line::from("(no selected record)"));
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

fn render_editor_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(ed) = &app.editor else {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Template Editor");
        frame.render_widget(Paragraph::new("(no editor loaded)").block(block), area);
        return;
    };

    let title = format!(
        "Template Editor{} (Ctrl+S save, Esc exit)",
        if ed.dirty { " *" } else { "" }
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    let view_height = inner.height as usize;

    let start = ed.buf.scroll_row.min(ed.buf.lines.len().saturating_sub(1));
    let end = (start + view_height).min(ed.buf.lines.len());

    let mut lines: Vec<Line> = Vec::new();
    for (i, line) in ed.buf.lines.iter().enumerate().take(end).skip(start) {
        let row = start + i;
        let is_cursor = row == ed.buf.cursor_row;
        let prefix = if is_cursor { "> " } else { "  " };
        let mut style = Style::default();
        if is_cursor {
            style = style
                .bg(Color::Rgb(50, 50, 50))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(line.as_str(), style),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from("(empty)"));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("file: {}", ed.path.display()),
        Style::default().fg(Color::Gray),
    )));

    if let Some(err) = &ed.last_save_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "save error:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        for l in err.lines().take(2) {
            lines.push(Line::from(Span::styled(l, Style::default().fg(Color::Red))));
        }
    }

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn render_timeline_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(report) = &app.last_good else {
        let block = Block::default().borders(Borders::ALL).title("Timeline");
        frame.render_widget(Paragraph::new("(no trace)").block(block), area);
        return;
    };

    // Apply filtering
    let filtered: Vec<(usize, &cliscrape::engine::debug::TraceEvent)> = report
        .trace
        .iter()
        .enumerate()
        .filter(|(_, e)| app.filter_state.matches(&e.event_type))
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|(idx, event)| {
            let is_current = *idx == app.trace_index;
            let prefix = if is_current { ">" } else { " " };

            // User decision: show line numbers with state (e.g., 'HEADER @L15')
            let state_display = if event.state_before == event.state_after {
                format!("{} @L{}", event.state_after, event.line_idx + 1)
            } else {
                format!(
                    "{} -> {} @L{}",
                    event.state_before,
                    event.state_after,
                    event.line_idx + 1
                )
            };

            let text = format!("{} {} | {:?}", prefix, state_display, event.event_type);

            // Highlight current event (user decision: visual distinction)
            let style = if is_current {
                Style::default()
                    .bg(Color::Rgb(50, 50, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let title = format!(
        "Timeline ({}/{}) | Mode: {:?}",
        app.trace_index + 1,
        report.trace.len(),
        app.stepping_mode
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_variables_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(report) = &app.last_good else {
        let block = Block::default().borders(Borders::ALL).title("Variables");
        frame.render_widget(Paragraph::new("(no trace)").block(block), area);
        return;
    };

    let Some(current_event) = report.trace.get(app.trace_index) else {
        let block = Block::default().borders(Borders::ALL).title("Variables");
        frame.render_widget(
            Paragraph::new("(trace index out of range)").block(block),
            area,
        );
        return;
    };

    let prev_event = if app.trace_index > 0 {
        report.trace.get(app.trace_index - 1)
    } else {
        None
    };

    let mut lines: Vec<Line> = Vec::new();

    // Sort variables: watched first, then alphabetical
    let mut var_names: Vec<&String> = current_event.variables.keys().collect();
    var_names.sort_by_key(|name| {
        let is_watched = app.watch_list.contains(*name);
        (!is_watched, *name) // watched sort first (false < true)
    });

    for var_name in var_names {
        let current_val = &current_event.variables[var_name];
        let prev_val = prev_event.and_then(|p| p.variables.get(var_name));

        let changed = prev_val.map(|p| p != current_val).unwrap_or(true);
        let is_watched = app.watch_list.contains(var_name);

        // User decision: highlight changed variables + show old->new
        let (text, style) = if changed {
            let change_text = if let Some(prev) = prev_val {
                format!(
                    "{} = {} -> {}",
                    var_name,
                    display_value_compact(prev),
                    display_value_compact(current_val)
                )
            } else {
                format!("{} = {} (new)", var_name, display_value_compact(current_val))
            };
            (
                change_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            let text = format!("{} = {}", var_name, display_value_compact(current_val));
            (text, Style::default())
        };

        // Add watch indicator
        let display = if is_watched {
            format!("★ {}", text)
        } else {
            format!("  {}", text)
        };

        lines.push(Line::from(Span::styled(display, style)));
    }

    if lines.is_empty() {
        lines.push(Line::from("(no variables at this trace point)"));
    }

    let title = format!("Variables @L{}", current_event.line_idx + 1);
    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
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

    let view_str = match app.view_mode {
        ViewMode::Matches => "matches",
        ViewMode::Records => "records",
    };
    lines.push(Line::from(format!("view:     {}", view_str)));

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

    let mode_str = match app.mode {
        Mode::Picker => "picker",
        Mode::Browse => "browse",
        Mode::EditTemplate => "edit-template",
    };
    lines.push(Line::from(format!("mode:     {}", mode_str)));

    if let Some(report) = &app.last_good {
        let cursor = app
            .cursor_line_idx
            .min(report.lines.len().saturating_sub(1));
        if app.view_mode == ViewMode::Matches {
            if let Some(sel_match) = report
                .matches_by_line
                .get(cursor)
                .and_then(|m| m.get(app.selected_match_idx))
            {
                let line = report.lines.get(cursor).map(|s| s.as_str()).unwrap_or("");
                let mut bad = 0usize;
                for c in &sel_match.captures {
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
