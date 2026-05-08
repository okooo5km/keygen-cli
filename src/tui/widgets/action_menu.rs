//! Floating menu listing the available actions for the selected row.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::{permission::Tier, views::actions::Action};

pub struct ActionMenuState {
    pub jsonapi_type: &'static str,
    pub resource_id: String,
    pub actions: &'static [Action],
    pub cursor: usize,
}

impl ActionMenuState {
    pub fn new(
        jsonapi_type: &'static str,
        resource_id: String,
        actions: &'static [Action],
    ) -> Self {
        Self {
            jsonapi_type,
            resource_id,
            actions,
            cursor: 0,
        }
    }

    pub fn selected(&self) -> Option<&'static Action> {
        self.actions.get(self.cursor)
    }

    pub fn move_down(&mut self) {
        if self.actions.is_empty() {
            return;
        }
        self.cursor = (self.cursor + 1) % self.actions.len();
    }

    pub fn move_up(&mut self) {
        if self.actions.is_empty() {
            return;
        }
        self.cursor = if self.cursor == 0 {
            self.actions.len() - 1
        } else {
            self.cursor - 1
        };
    }

    pub fn jump_to_key(&mut self, ch: char) -> bool {
        if let Some(idx) = self.actions.iter().position(|a| a.key == ch) {
            self.cursor = idx;
            true
        } else {
            false
        }
    }
}

pub fn draw(f: &mut Frame, area: Rect, state: &ActionMenuState) {
    let popup = centered(area, 60, 70);
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(popup);

    let title = format!(
        " actions for {} ({}) ",
        state.jsonapi_type, state.resource_id
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = state
        .actions
        .iter()
        .map(|a| {
            let tier_tag = match a.tier {
                Tier::AutoRun => Span::styled("[1] ", Style::default().fg(Color::Green)),
                Tier::DryRunConfirm => Span::styled("[2] ", Style::default().fg(Color::Yellow)),
                Tier::Explicit => Span::styled(
                    "[3] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            };
            let key = Span::styled(
                format!(" {} ", a.key),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let label = Span::styled(
                format!("  {:<18}", a.label),
                Style::default().add_modifier(Modifier::BOLD),
            );
            let hint = Span::styled(a.hint, Style::default().fg(Color::DarkGray));
            ListItem::new(Line::from(vec![tier_tag, key, label, Span::raw(" "), hint]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut ls = ListState::default();
    ls.select(Some(state.cursor));
    f.render_stateful_widget(list, chunks[0], &mut ls);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("↑↓/jk", Style::default().fg(Color::Cyan)),
        Span::raw(" move  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" run  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(hint, chunks[1]);
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
