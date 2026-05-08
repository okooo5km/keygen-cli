//! Data-driven view config: which columns each resource shows and how they
//! render. Both the CLI table renderer and the TUI list/detail panes consume
//! the same `ResourceView` so the look stays consistent.
//!
//! Authored by okooo5km(十里).

pub mod cards;
pub mod columns;
pub mod render;
pub mod truncate;

pub use cards::card_block;
pub use columns::{view_for_jsonapi_type, ColKind, ColumnDef, ColumnWidth, ResourceView};
pub use render::{cell_text, detail_pairs, format_value};
pub use truncate::truncate_middle;
