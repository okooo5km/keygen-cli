//! Scrolling log viewer used by the events panel.
//!
//! Renders newest-first, with a brief flash colour for rows that just
//! arrived. The flash flag is consumed by the caller after the first redraw
//! by setting `fresh = false`.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::views::events::EventEntry;

pub fn draw(f: &mut Frame, area: Rect, rows: &[EventEntry], focused: bool, error: Option<&str>) {
    let title = if focused {
        " events  ·  e=collapse  End=jump-latest ".to_string()
    } else {
        " events  ·  e=expand ".to_string()
    };
    let border = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(title);

    if let Some(err) = error {
        let p = Paragraph::new(err)
            .style(Style::default().fg(Color::Red))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    if rows.is_empty() {
        let p = Paragraph::new(
            "(no webhook events yet — wired? configure an endpoint to see deliveries here)",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = rows
        .iter()
        .map(|e| {
            let stamp = trim_to(&e.created, 19); // "YYYY-MM-DDTHH:MM:SS"
            let stamp_style = if e.fresh {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let kind_style = match e.status.as_deref() {
                Some("DELIVERED") => Style::default().fg(Color::Green),
                Some("FAILED") => Style::default().fg(Color::Red),
                Some("RETRYING") => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::White),
            };
            let status = e.status.as_deref().unwrap_or("--");
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {stamp} "), stamp_style),
                Span::raw(" "),
                Span::styled(format!("{status:<10}"), kind_style),
                Span::raw(" "),
                Span::styled(
                    e.event_type.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(e.id.as_str(), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn trim_to(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect()
    }
}
