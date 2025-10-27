use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::editor::Editor;

pub fn render_ui(f: &mut Frame, editor: &Editor) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(f.area());

    render_input_panel(f, editor, chunks[0]);
    render_results_panel(f, editor, chunks[1]);
}

fn render_input_panel(f: &mut Frame, editor: &Editor, area: Rect) {
    let mut lines: Vec<Line> = editor
        .lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let style = if idx == editor.cursor_line {
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))  // Bright white
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(150, 150, 150))  // Medium gray
            };
            Line::from(Span::styled(line.clone(), style))
        })
        .collect();

    // Add cursor indicator
    if editor.cursor_line < lines.len() {
        let cursor_line = &editor.lines[editor.cursor_line];
        let before_cursor = &cursor_line[..editor.cursor_col];
        let at_cursor = cursor_line
            .chars()
            .nth(editor.cursor_col)
            .unwrap_or(' ');
        let after_cursor = &cursor_line[editor.cursor_col.min(cursor_line.len())..];

        lines[editor.cursor_line] = Line::from(vec![
            Span::styled(
                before_cursor.to_string(),
                Style::default().fg(Color::Rgb(255, 255, 255))
            ),
            Span::styled(
                at_cursor.to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                after_cursor.to_string(),
                Style::default().fg(Color::Rgb(255, 255, 255))
            ),
        ]);
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Indumi Calculator (Ctrl+C to quit)")
                .border_style(Style::default().fg(Color::Rgb(0, 255, 255)))  // Bright cyan
                .title_style(Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        );

    f.render_widget(paragraph, area);
}

fn render_results_panel(f: &mut Frame, editor: &Editor, area: Rect) {
    let results: Vec<Line> = editor
        .lines
        .iter()
        .map(|line| {
            if let Some(result) = editor.calculator.borrow_mut().evaluate_line(line) {
                // Check if result is an error
                let (text, color) = if result.starts_with("Error:") || result.starts_with("Parse error:") {
                    (format!("= {}", result), Color::Rgb(255, 80, 80))  // Bright red for errors
                } else {
                    (format!("= {}", result), Color::Rgb(0, 255, 0))  // Bright green for results
                };

                Line::from(Span::styled(
                    text,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled("", Style::default()))
            }
        })
        .collect();

    let paragraph = Paragraph::new(results)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results")
                .border_style(Style::default().fg(Color::Rgb(255, 0, 255)))  // Bright magenta
                .title_style(Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)),
        );

    f.render_widget(paragraph, area);
}
