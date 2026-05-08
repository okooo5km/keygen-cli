//! Vim-style `:` command input bar.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{permission::Tier, views::command_palette::PaletteState};

pub fn draw(f: &mut Frame, area: Rect, state: &PaletteState) {
    let popup = centered(area, 80, 70);
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // input
            Constraint::Min(3),    // suggestions
            Constraint::Min(3),    // result
            Constraint::Length(2), // hint
        ])
        .split(popup);

    draw_input(f, chunks[0], state);
    draw_suggestions(f, chunks[1], state);
    draw_result(f, chunks[2], state);
    draw_hint(f, chunks[3]);
}

fn draw_input(f: &mut Frame, area: Rect, state: &PaletteState) {
    let title = " : command palette ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let prefix = ":";
    let value = &state.input;
    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(
            value.as_str(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "▌",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]);
    let p = Paragraph::new(line).block(block);
    f.render_widget(p, area);
}

fn draw_suggestions(f: &mut Frame, area: Rect, state: &PaletteState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tab to complete ")
        .border_style(Style::default().fg(Color::DarkGray));

    if state.suggestions.is_empty() {
        let p = Paragraph::new("(no completions for current token)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = state
        .suggestions
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(s.as_str(), Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_result(f: &mut Frame, area: Rect, state: &PaletteState) {
    if let Some(aw) = &state.awaiting {
        let (style, banner_text) = match aw.tier {
            Tier::Explicit => (
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                " ⚠ DESTRUCTIVE / IRREVERSIBLE — y to run, n / Esc to cancel ",
            ),
            _ => (
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                " confirm — y to run, n / Esc to cancel ",
            ),
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" pending — awaiting confirmation ")
            .border_style(style);
        let lines = vec![
            Line::from(Span::styled(banner_text, style)),
            Line::raw(""),
            Line::from(vec![
                Span::styled("$ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    aw.display.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let p = Paragraph::new(lines).block(block);
        f.render_widget(p, area);
        return;
    }

    let title = if state.in_flight {
        " ⟳ executing… ".to_string()
    } else {
        " result ".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(title);

    let text: ratatui::text::Text = if let Some(err) = &state.error {
        Line::from(Span::styled(err.as_str(), Style::default().fg(Color::Red))).into()
    } else if let Some(out) = &state.output {
        out.as_str().into()
    } else {
        Line::from(Span::styled(
            "(Enter to run, Esc to close)",
            Style::default().fg(Color::DarkGray),
        ))
        .into()
    };

    let p = Paragraph::new(text)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_hint(f: &mut Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" run  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" complete  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" close"),
    ]))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(hint, area);
}

fn centered(area: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(v[1])[1]
}
