//! Per-resource column definitions.
//!
//! A `ResourceView` declares:
//! - the columns to show in list mode (`columns`),
//! - the field order to show in the detail / KV pane (`detail`).
//!
//! Every column is identified by a JSON pointer into the resource document so
//! both the table renderer and the TUI detail pane can pull the same value.
//!
//! Authored by okooo5km(十里).

#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    /// Fixed cell width.
    Fixed(u16),
    /// Minimum cell width — grows when room is available.
    Min(u16),
    /// Percentage of the available row width.
    Pct(u16),
}

#[derive(Debug, Clone, Copy)]
pub enum ColKind {
    /// Render the value as plain text.
    Plain,
    /// Render with `Status::pill`.
    Status,
    /// Parse as ISO8601 → relative time.
    Time,
    /// Take last N chars of the value (key, fingerprint).
    Tail(usize),
    /// Format a byte count.
    Bytes,
    /// Bool → ✓ / —.
    Bool,
    /// JSON:API relationships meta count: `meta.count` at the pointer.
    Count,
    /// Truncate using unicode-width middle ellipsis.
    Truncate(usize),
    /// Take the URL host portion only.
    UrlHost,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnDef {
    pub title: &'static str,
    /// JSON pointer relative to the resource (root). e.g. `/attributes/name`.
    /// Multiple alternatives separated by `|` are tried in order.
    pub pointer: &'static str,
    pub width: ColumnWidth,
    pub kind: ColKind,
    /// Hint that the value should never wrap (typically the ID column).
    pub no_wrap: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DetailField {
    pub label: &'static str,
    pub pointer: &'static str,
    pub kind: ColKind,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceView {
    pub jsonapi_type: &'static str,
    pub columns: &'static [ColumnDef],
    pub detail: &'static [DetailField],
}

const fn col(
    title: &'static str,
    pointer: &'static str,
    width: ColumnWidth,
    kind: ColKind,
) -> ColumnDef {
    ColumnDef {
        title,
        pointer,
        width,
        kind,
        no_wrap: false,
    }
}

const fn id_col() -> ColumnDef {
    ColumnDef {
        title: "id",
        pointer: "/id",
        width: ColumnWidth::Fixed(36),
        kind: ColKind::Plain,
        no_wrap: true,
    }
}

const fn d(label: &'static str, pointer: &'static str, kind: ColKind) -> DetailField {
    DetailField {
        label,
        pointer,
        kind,
    }
}

// ---------- license ----------

const LICENSE_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name|/attributes/key",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
    col(
        "expiry",
        "/attributes/expiry",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
    col(
        "machines",
        "/relationships/machines",
        ColumnWidth::Fixed(8),
        ColKind::Count,
    ),
];
const LICENSE_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("key", "/attributes/key", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Status),
    d("scheme", "/attributes/scheme", ColKind::Plain),
    d("suspended", "/attributes/suspended", ColKind::Bool),
    d("expiry", "/attributes/expiry", ColKind::Time),
    d("lastValidated", "/attributes/lastValidated", ColKind::Time),
    d("uses", "/attributes/uses", ColKind::Plain),
    d("maxMachines", "/attributes/maxMachines", ColKind::Plain),
    d("maxCores", "/attributes/maxCores", ColKind::Plain),
    d("maxUses", "/attributes/maxUses", ColKind::Plain),
    d(
        "requireCheckIn",
        "/attributes/requireCheckIn",
        ColKind::Bool,
    ),
    d("created", "/attributes/created", ColKind::Time),
    d("updated", "/attributes/updated", ColKind::Time),
    d("machines", "/relationships/machines", ColKind::Count),
    d("policy", "/relationships/policy/data/id", ColKind::Plain),
    d("user", "/relationships/user/data/id", ColKind::Plain),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- machine ----------

const MACHINE_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "fingerprint",
        "/attributes/fingerprint",
        ColumnWidth::Fixed(14),
        ColKind::Tail(12),
    ),
    col(
        "name",
        "/attributes/name|/attributes/hostname",
        ColumnWidth::Min(15),
        ColKind::Truncate(30),
    ),
    col(
        "platform",
        "/attributes/platform",
        ColumnWidth::Fixed(12),
        ColKind::Plain,
    ),
    col(
        "lastHeartbeat",
        "/attributes/lastHeartbeat",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const MACHINE_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("fingerprint", "/attributes/fingerprint", ColKind::Plain),
    d("platform", "/attributes/platform", ColKind::Plain),
    d("hostname", "/attributes/hostname", ColKind::Plain),
    d("ip", "/attributes/ip", ColKind::Plain),
    d("cores", "/attributes/cores", ColKind::Plain),
    d("lastHeartbeat", "/attributes/lastHeartbeat", ColKind::Time),
    d(
        "heartbeatStatus",
        "/attributes/heartbeatStatus",
        ColKind::Status,
    ),
    d("license", "/relationships/license/data/id", ColKind::Plain),
    d("components", "/relationships/components", ColKind::Count),
    d("processes", "/relationships/processes", ColKind::Count),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- policy ----------

const POLICY_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "scheme",
        "/attributes/scheme",
        ColumnWidth::Fixed(16),
        ColKind::Plain,
    ),
    col(
        "duration",
        "/attributes/duration",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "maxMachines",
        "/attributes/maxMachines",
        ColumnWidth::Fixed(12),
        ColKind::Plain,
    ),
    col(
        "floating",
        "/attributes/floating",
        ColumnWidth::Fixed(8),
        ColKind::Bool,
    ),
];
const POLICY_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("scheme", "/attributes/scheme", ColKind::Plain),
    d("duration", "/attributes/duration", ColKind::Plain),
    d("maxMachines", "/attributes/maxMachines", ColKind::Plain),
    d("maxCores", "/attributes/maxCores", ColKind::Plain),
    d("maxUsers", "/attributes/maxUsers", ColKind::Plain),
    d("maxUses", "/attributes/maxUses", ColKind::Plain),
    d("floating", "/attributes/floating", ColKind::Bool),
    d("strict", "/attributes/strict", ColKind::Bool),
    d(
        "requireCheckIn",
        "/attributes/requireCheckIn",
        ColKind::Bool,
    ),
    d(
        "checkInInterval",
        "/attributes/checkInInterval",
        ColKind::Plain,
    ),
    d(
        "requireHeartbeat",
        "/attributes/requireHeartbeat",
        ColKind::Bool,
    ),
    d(
        "heartbeatDuration",
        "/attributes/heartbeatDuration",
        ColKind::Plain,
    ),
    d("product", "/relationships/product/data/id", ColKind::Plain),
    d(
        "entitlements",
        "/relationships/entitlements",
        ColKind::Count,
    ),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- product ----------

const PRODUCT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "code",
        "/attributes/code",
        ColumnWidth::Min(12),
        ColKind::Plain,
    ),
    col(
        "distribution",
        "/attributes/distributionStrategy",
        ColumnWidth::Fixed(13),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const PRODUCT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("code", "/attributes/code", ColKind::Plain),
    d(
        "distributionStrategy",
        "/attributes/distributionStrategy",
        ColKind::Plain,
    ),
    d("url", "/attributes/url", ColKind::Plain),
    d("platforms", "/attributes/platforms", ColKind::Plain),
    d("permissions", "/attributes/permissions", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
    d("updated", "/attributes/updated", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- user ----------

const USER_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/fullName|/attributes/email",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "email",
        "/attributes/email",
        ColumnWidth::Min(20),
        ColKind::Plain,
    ),
    col(
        "role",
        "/attributes/role",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
    col(
        "banned",
        "/attributes/banned",
        ColumnWidth::Fixed(7),
        ColKind::Bool,
    ),
];
const USER_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("fullName", "/attributes/fullName", ColKind::Plain),
    d("firstName", "/attributes/firstName", ColKind::Plain),
    d("lastName", "/attributes/lastName", ColKind::Plain),
    d("email", "/attributes/email", ColKind::Plain),
    d("role", "/attributes/role", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Status),
    d("banned", "/attributes/banned", ColKind::Plain),
    d("lastLogin", "/attributes/lastLogin", ColKind::Time),
    d("created", "/attributes/created", ColKind::Time),
    d("groups", "/relationships/groups", ColKind::Count),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- group ----------

const GROUP_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "maxLicenses",
        "/attributes/maxLicenses",
        ColumnWidth::Fixed(12),
        ColKind::Plain,
    ),
    col(
        "maxMachines",
        "/attributes/maxMachines",
        ColumnWidth::Fixed(12),
        ColKind::Plain,
    ),
    col(
        "maxUsers",
        "/attributes/maxUsers",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const GROUP_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("maxLicenses", "/attributes/maxLicenses", ColKind::Plain),
    d("maxMachines", "/attributes/maxMachines", ColKind::Plain),
    d("maxUsers", "/attributes/maxUsers", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- release ----------

const RELEASE_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "version",
        "/attributes/version",
        ColumnWidth::Fixed(14),
        ColKind::Plain,
    ),
    col(
        "channel",
        "/attributes/channel",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
    col(
        "yanked",
        "/attributes/yanked",
        ColumnWidth::Fixed(7),
        ColKind::Bool,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const RELEASE_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("version", "/attributes/version", ColKind::Plain),
    d("channel", "/attributes/channel", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Status),
    d("yanked", "/attributes/yanked", ColKind::Bool),
    d("yankedAt", "/attributes/yankedAt", ColKind::Time),
    d("tag", "/attributes/tag", ColKind::Plain),
    d("description", "/attributes/description", ColKind::Plain),
    d("product", "/relationships/product/data/id", ColKind::Plain),
    d("package", "/relationships/package/data/id", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- artifact ----------

const ARTIFACT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "filename",
        "/attributes/filename",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "platform",
        "/attributes/platform",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "arch",
        "/attributes/arch",
        ColumnWidth::Fixed(8),
        ColKind::Plain,
    ),
    col(
        "filesize",
        "/attributes/filesize",
        ColumnWidth::Fixed(10),
        ColKind::Bytes,
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
];
const ARTIFACT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("filename", "/attributes/filename", ColKind::Plain),
    d("filesize", "/attributes/filesize", ColKind::Bytes),
    d("filetype", "/attributes/filetype", ColKind::Plain),
    d("platform", "/attributes/platform", ColKind::Plain),
    d("arch", "/attributes/arch", ColKind::Plain),
    d("signature", "/attributes/signature", ColKind::Plain),
    d("checksum", "/attributes/checksum", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Status),
    d("release", "/relationships/release/data/id", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- package ----------

const PACKAGE_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "key",
        "/attributes/key",
        ColumnWidth::Min(12),
        ColKind::Plain,
    ),
    col(
        "engine",
        "/attributes/engine",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const PACKAGE_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("key", "/attributes/key", ColKind::Plain),
    d("engine", "/attributes/engine", ColKind::Plain),
    d("product", "/relationships/product/data/id", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- component ----------

const COMPONENT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "fingerprint",
        "/attributes/fingerprint",
        ColumnWidth::Fixed(10),
        ColKind::Tail(8),
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const COMPONENT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("fingerprint", "/attributes/fingerprint", ColKind::Plain),
    d("machine", "/relationships/machine/data/id", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- token ----------

const TOKEN_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "kind",
        "/attributes/kind",
        ColumnWidth::Fixed(14),
        ColKind::Plain,
    ),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(15),
        ColKind::Truncate(30),
    ),
    col(
        "expiry",
        "/attributes/expiry",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const TOKEN_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("kind", "/attributes/kind", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("token", "/attributes/token", ColKind::Plain),
    d("expiry", "/attributes/expiry", ColKind::Time),
    d(
        "maxActivations",
        "/attributes/maxActivations",
        ColKind::Plain,
    ),
    d("activations", "/attributes/activations", ColKind::Plain),
    d("permissions", "/attributes/permissions", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- process ----------

const PROCESS_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "pid",
        "/attributes/pid",
        ColumnWidth::Fixed(10),
        ColKind::Plain,
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
    col(
        "lastHeartbeat",
        "/attributes/lastHeartbeat",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const PROCESS_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("pid", "/attributes/pid", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Status),
    d("lastHeartbeat", "/attributes/lastHeartbeat", ColKind::Time),
    d(
        "heartbeatStatus",
        "/attributes/heartbeatStatus",
        ColKind::Status,
    ),
    d("machine", "/relationships/machine/data/id", ColKind::Plain),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- entitlement ----------

const ENTITLEMENT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "name",
        "/attributes/name",
        ColumnWidth::Min(20),
        ColKind::Truncate(40),
    ),
    col(
        "code",
        "/attributes/code",
        ColumnWidth::Min(12),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const ENTITLEMENT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("name", "/attributes/name", ColKind::Plain),
    d("code", "/attributes/code", ColKind::Plain),
    d("metadata", "/attributes/metadata", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- webhook endpoint ----------

const WEBHOOK_ENDPOINT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "url",
        "/attributes/url",
        ColumnWidth::Min(20),
        ColKind::UrlHost,
    ),
    col(
        "subscriptions",
        "/attributes/subscriptions",
        ColumnWidth::Fixed(13),
        ColKind::Plain,
    ),
    col(
        "signatureAlgorithm",
        "/attributes/signatureAlgorithm",
        ColumnWidth::Fixed(18),
        ColKind::Plain,
    ),
    col(
        "enabled",
        "/attributes/enabled",
        ColumnWidth::Fixed(8),
        ColKind::Bool,
    ),
];
const WEBHOOK_ENDPOINT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("url", "/attributes/url", ColKind::Plain),
    d("subscriptions", "/attributes/subscriptions", ColKind::Plain),
    d(
        "signatureAlgorithm",
        "/attributes/signatureAlgorithm",
        ColKind::Plain,
    ),
    d("apiVersion", "/attributes/apiVersion", ColKind::Plain),
    d("enabled", "/attributes/enabled", ColKind::Bool),
    d("created", "/attributes/created", ColKind::Time),
    d("metadata", "/attributes/metadata", ColKind::Plain),
];

// ---------- webhook event ----------

const WEBHOOK_EVENT_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "event",
        "/attributes/event",
        ColumnWidth::Min(20),
        ColKind::Plain,
    ),
    col(
        "endpoint",
        "/relationships/endpoint/data/id",
        ColumnWidth::Fixed(36),
        ColKind::Plain,
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(11),
        ColKind::Status,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const WEBHOOK_EVENT_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("event", "/attributes/event", ColKind::Plain),
    d(
        "endpoint",
        "/relationships/endpoint/data/id",
        ColKind::Plain,
    ),
    d("status", "/attributes/status", ColKind::Status),
    d(
        "lastResponseCode",
        "/attributes/lastResponseCode",
        ColKind::Plain,
    ),
    d(
        "lastResponseBody",
        "/attributes/lastResponseBody",
        ColKind::Plain,
    ),
    d("payload", "/attributes/payload", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- event-log ----------

const EVENT_LOG_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "event",
        "/attributes/event",
        ColumnWidth::Min(20),
        ColKind::Plain,
    ),
    col(
        "whodunnit",
        "/relationships/whodunnit/data/id",
        ColumnWidth::Fixed(36),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const EVENT_LOG_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("event", "/attributes/event", ColKind::Plain),
    d("whodunnit", "/relationships/whodunnit/data", ColKind::Plain),
    d("resource", "/relationships/resource/data", ColKind::Plain),
    d("changes", "/attributes/changes", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- request-log ----------

const REQUEST_LOG_COLS: &[ColumnDef] = &[
    id_col(),
    col(
        "method",
        "/attributes/method",
        ColumnWidth::Fixed(8),
        ColKind::Plain,
    ),
    col(
        "url",
        "/attributes/url",
        ColumnWidth::Min(20),
        ColKind::Truncate(60),
    ),
    col(
        "status",
        "/attributes/status",
        ColumnWidth::Fixed(8),
        ColKind::Plain,
    ),
    col(
        "created",
        "/attributes/created",
        ColumnWidth::Fixed(18),
        ColKind::Time,
    ),
];
const REQUEST_LOG_DETAIL: &[DetailField] = &[
    d("id", "/id", ColKind::Plain),
    d("method", "/attributes/method", ColKind::Plain),
    d("url", "/attributes/url", ColKind::Plain),
    d("status", "/attributes/status", ColKind::Plain),
    d("ip", "/attributes/ip", ColKind::Plain),
    d("userAgent", "/attributes/userAgent", ColKind::Plain),
    d("requestBody", "/attributes/requestBody", ColKind::Plain),
    d("responseBody", "/attributes/responseBody", ColKind::Plain),
    d("created", "/attributes/created", ColKind::Time),
];

// ---------- registry ----------

pub const VIEWS: &[ResourceView] = &[
    ResourceView {
        jsonapi_type: "licenses",
        columns: LICENSE_COLS,
        detail: LICENSE_DETAIL,
    },
    ResourceView {
        jsonapi_type: "machines",
        columns: MACHINE_COLS,
        detail: MACHINE_DETAIL,
    },
    ResourceView {
        jsonapi_type: "policies",
        columns: POLICY_COLS,
        detail: POLICY_DETAIL,
    },
    ResourceView {
        jsonapi_type: "products",
        columns: PRODUCT_COLS,
        detail: PRODUCT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "users",
        columns: USER_COLS,
        detail: USER_DETAIL,
    },
    ResourceView {
        jsonapi_type: "groups",
        columns: GROUP_COLS,
        detail: GROUP_DETAIL,
    },
    ResourceView {
        jsonapi_type: "releases",
        columns: RELEASE_COLS,
        detail: RELEASE_DETAIL,
    },
    ResourceView {
        jsonapi_type: "artifacts",
        columns: ARTIFACT_COLS,
        detail: ARTIFACT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "packages",
        columns: PACKAGE_COLS,
        detail: PACKAGE_DETAIL,
    },
    ResourceView {
        jsonapi_type: "components",
        columns: COMPONENT_COLS,
        detail: COMPONENT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "tokens",
        columns: TOKEN_COLS,
        detail: TOKEN_DETAIL,
    },
    ResourceView {
        jsonapi_type: "processes",
        columns: PROCESS_COLS,
        detail: PROCESS_DETAIL,
    },
    ResourceView {
        jsonapi_type: "entitlements",
        columns: ENTITLEMENT_COLS,
        detail: ENTITLEMENT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "webhook-endpoints",
        columns: WEBHOOK_ENDPOINT_COLS,
        detail: WEBHOOK_ENDPOINT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "webhook-events",
        columns: WEBHOOK_EVENT_COLS,
        detail: WEBHOOK_EVENT_DETAIL,
    },
    ResourceView {
        jsonapi_type: "event-logs",
        columns: EVENT_LOG_COLS,
        detail: EVENT_LOG_DETAIL,
    },
    ResourceView {
        jsonapi_type: "request-logs",
        columns: REQUEST_LOG_COLS,
        detail: REQUEST_LOG_DETAIL,
    },
];

/// Look up the per-resource view for a JSON:API type. Returns `None` if the
/// type is unknown — caller falls back to a generic ID + attrs view.
pub fn view_for_jsonapi_type(t: &str) -> Option<&'static ResourceView> {
    VIEWS.iter().find(|v| v.jsonapi_type == t)
}
