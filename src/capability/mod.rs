pub mod detect;
pub mod doctor;
pub mod env_commands;

#[derive(Debug, Default, Clone)]
pub struct Capabilities {
    pub environments: bool,
    pub event_logs: bool,
    pub request_logs: bool,
    pub sso: bool,
    pub oci_registry: bool,
    pub import_export: bool,
    /// Whether `filter[<relation>]=<id>` is honored by the server. `None`
    /// means we have not probed (or the probe failed); `Some(true)` means
    /// the server applied the filter; `Some(false)` means it was ignored.
    pub filters_relation: Option<bool>,
}
