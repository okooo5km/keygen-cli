//! Shared TUI state.
//!
//! Lives outside `app.rs` so view modules under `tui/views` can read and
//! mutate the dashboard's state without coupling the event loop to render
//! code.
//!
//! Authored by okooo5km.

use ratatui::widgets::TableState;
use serde_json::Value;

use crate::{
    api::jsonapi::Resource,
    tui::{
        views::{command_palette::PaletteState, events::EventEntry},
        widgets::{action_menu::ActionMenuState, confirm::ConfirmState},
    },
    view::columns::ResourceView,
};

/// (label, jsonapi-type, list-path)
pub const RESOURCES: &[(&str, &str, &str)] = &[
    ("Licenses", "licenses", "/licenses"),
    ("Machines", "machines", "/machines"),
    ("Policies", "policies", "/policies"),
    ("Products", "products", "/products"),
    ("Users", "users", "/users"),
    ("Groups", "groups", "/groups"),
    ("Releases", "releases", "/releases"),
    ("Artifacts", "artifacts", "/artifacts"),
    ("Webhooks", "webhook-endpoints", "/webhook-endpoints"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Split,
    DetailFull,
    Cards,
    EventsFull,
}

pub struct AppState {
    pub selected_resource: usize,
    pub table_state: TableState,
    pub rows: Vec<Resource>,
    pub error: Option<String>,
    pub loading: bool,
    pub status: String,
    pub layout: LayoutMode,
    pub detail_cursor: usize,
    pub flash: Option<String>,
    pub fetch_seq: u64,
    pub action_menu: Option<ActionMenuState>,
    pub confirm: Option<ConfirmState>,
    pub events: Vec<EventEntry>,
    pub events_cursor: Option<String>,
    pub events_error: Option<String>,
    pub events_fetching: bool,
    pub palette: Option<PaletteState>,
}

impl AppState {
    pub fn new(default_layout: LayoutMode) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            selected_resource: 0,
            table_state: state,
            rows: Vec::new(),
            error: None,
            loading: false,
            status:
                "Ready. Tab switch · a=actions · d=detail · c=cards · e=events · y=yank · q=quit"
                    .into(),
            layout: default_layout,
            detail_cursor: 0,
            flash: None,
            fetch_seq: 0,
            action_menu: None,
            confirm: None,
            events: Vec::new(),
            events_cursor: None,
            events_error: None,
            events_fetching: false,
            palette: None,
        }
    }

    pub fn selected_id(&self) -> Option<String> {
        let i = self.table_state.selected()?;
        self.rows.get(i).map(|r| r.id.clone())
    }

    pub fn jsonapi_type(&self) -> &'static str {
        RESOURCES[self.selected_resource].1
    }

    pub fn selected_value(&self) -> Option<Value> {
        let i = self.table_state.selected()?;
        self.rows.get(i).map(resource_to_value)
    }

    pub fn move_left(&mut self) {
        if self.selected_resource > 0 {
            self.selected_resource -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected_resource + 1 < RESOURCES.len() {
            self.selected_resource += 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| (i + 1) % self.rows.len());
        self.table_state.select(Some(i));
        self.detail_cursor = 0;
    }

    pub fn move_up(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i =
            self.table_state
                .selected()
                .map_or(0, |i| if i == 0 { self.rows.len() - 1 } else { i - 1 });
        self.table_state.select(Some(i));
        self.detail_cursor = 0;
    }

    pub fn detail_down(&mut self, view: Option<&'static ResourceView>) {
        if let Some(rv) = view {
            if rv.detail.is_empty() {
                return;
            }
            self.detail_cursor = (self.detail_cursor + 1) % rv.detail.len();
        }
    }

    pub fn detail_up(&mut self, view: Option<&'static ResourceView>) {
        if let Some(rv) = view {
            if rv.detail.is_empty() {
                return;
            }
            self.detail_cursor = if self.detail_cursor == 0 {
                rv.detail.len() - 1
            } else {
                self.detail_cursor - 1
            };
        }
    }
}

pub fn resource_to_value(r: &Resource) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), Value::String(r.id.clone()));
    obj.insert("type".into(), Value::String(r.r#type.clone()));
    obj.insert("attributes".into(), r.attributes.clone());
    if let Some(rels) = &r.relationships {
        obj.insert("relationships".into(), rels.clone());
    }
    Value::Object(obj)
}

pub fn current_view(state: &AppState) -> Option<&'static ResourceView> {
    let t = RESOURCES[state.selected_resource].1;
    crate::view::view_for_jsonapi_type(t)
}

pub struct FetchResult {
    pub seq: u64,
    pub resource_idx: usize,
    pub payload: std::result::Result<Vec<Resource>, String>,
}

pub struct ActionDone {
    pub label: &'static str,
    pub payload: std::result::Result<Value, String>,
}

pub enum AppMsg {
    Fetch(FetchResult),
    Action(ActionDone),
    Events(std::result::Result<Vec<EventEntry>, String>),
    Shell(std::result::Result<String, String>),
}
