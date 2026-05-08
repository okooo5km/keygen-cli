use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};

#[derive(Debug, Clone)]
pub struct Column {
    pub key: &'static str,
    pub title: &'static str,
}

pub fn render(columns: &[Column], rows: &[Vec<String>]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(columns.iter().map(|c| Cell::new(c.title)));
    for row in rows {
        table.add_row(row.iter().map(Cell::new));
    }
    table
}
