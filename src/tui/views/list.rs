//! Master list — table and card layouts.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell as TCell, Paragraph, Row, Table},
    Frame,
};

use crate::{
    tui::state::{resource_to_value, AppState, RESOURCES},
    view::{
        self,
        cards::card_block,
        columns::{ColumnWidth, ResourceView},
        ColumnDef,
    },
};

pub fn draw_table(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    view: Option<&'static ResourceView>,
    label: &str,
) {
    let cols: Vec<ColumnDef> = view.map_or_else(
        || crate::view::columns::VIEWS[0].columns.to_vec(),
        |v| v.columns.to_vec(),
    );

    let header = Row::new(cols.iter().map(|c| {
        TCell::from(c.title).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    }));

    let rows: Vec<Row> = state
        .rows
        .iter()
        .map(|r| {
            let v = resource_to_value(r);
            let cells: Vec<TCell> = cols
                .iter()
                .map(|c| {
                    let text = view::cell_text(&v, c, false);
                    let style = column_style(c.title);
                    TCell::from(text).style(style)
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let widths: Vec<Constraint> = cols.iter().map(|c| width_to_constraint(c.width)).collect();

    let title = format!(
        " {label} ({n}) — d=detail c=cards r=refresh q=quit ",
        n = state.rows.len(),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(table, area, &mut state.table_state);
}

pub fn draw_cards(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    view: Option<&'static ResourceView>,
    label: &str,
) {
    let title = format!(
        " {label} cards (n={n}) — c=table r=refresh q=quit ",
        n = state.rows.len(),
    );
    let block = Block::default().borders(Borders::ALL).title(title);

    let Some(rv) = view else {
        let p = Paragraph::new("(card view unavailable for this resource)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    };

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut text = String::new();
    let inner_w = inner.width as usize;
    for r in &state.rows {
        let v = resource_to_value(r);
        text.push_str(&card_block(&v, rv, false, inner_w));
        text.push('\n');
    }
    let p = Paragraph::new(text);
    f.render_widget(p, inner);
}

pub fn current_resource_label(state: &AppState) -> &'static str {
    RESOURCES[state.selected_resource].0
}

fn width_to_constraint(w: ColumnWidth) -> Constraint {
    match w {
        ColumnWidth::Fixed(n) => Constraint::Length(n),
        ColumnWidth::Min(n) => Constraint::Min(n),
        ColumnWidth::Pct(n) => Constraint::Percentage(n),
    }
}

fn column_style(title: &str) -> Style {
    match title {
        "id" => Style::default().fg(Color::Yellow),
        "created" | "expiry" | "lastHeartbeat" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}
