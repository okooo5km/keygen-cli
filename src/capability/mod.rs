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
}
