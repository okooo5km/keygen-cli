use clap::Subcommand;

use crate::resources;

/// Resource sub-trees. Each resource has its own CRUD + action subcommand
/// surface defined in `crate::resources::<name>::commands`.
#[derive(Debug, Subcommand)]
pub enum ResourceCommand {
    /// API tokens.
    #[command(subcommand)]
    Token(resources::token::Cmd),
    /// Products.
    #[command(subcommand)]
    Product(resources::product::Cmd),
    /// Policies.
    #[command(subcommand)]
    Policy(resources::policy::Cmd),
    /// Licenses.
    #[command(subcommand)]
    License(resources::license::Cmd),
    /// Entitlements.
    #[command(subcommand)]
    Entitlement(resources::entitlement::Cmd),
    /// Users.
    #[command(subcommand)]
    User(resources::user::Cmd),
    /// Groups.
    #[command(subcommand)]
    Group(resources::group::Cmd),
    /// Machines (license activations).
    #[command(subcommand)]
    Machine(resources::machine::Cmd),
    /// Machine components (hardware fingerprints).
    #[command(subcommand)]
    Component(resources::component::Cmd),
    /// Machine processes (heartbeats).
    #[command(subcommand)]
    Process(resources::process::Cmd),
    /// Releases.
    #[command(subcommand)]
    Release(resources::release::Cmd),
    /// Release artifacts (binaries).
    #[command(subcommand)]
    Artifact(resources::artifact::Cmd),
    /// Release packages.
    #[command(subcommand)]
    Package(resources::package::Cmd),
    /// Webhook endpoints + events.
    #[command(subcommand)]
    Webhook(resources::webhook::Cmd),
    /// EE-only: request logs.
    #[command(name = "request-log", subcommand)]
    RequestLog(resources::request_log::Cmd),
    /// EE-only: event logs.
    #[command(name = "event-log", subcommand)]
    EventLog(resources::event_log::Cmd),
}
