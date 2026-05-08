use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Active,
    Expiring,
    Expired,
    Suspended,
    Banned,
    Revoked,
    Inactive,
    Unknown,
}

impl Status {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "ACTIVE" => Self::Active,
            "EXPIRING" => Self::Expiring,
            "EXPIRED" => Self::Expired,
            "SUSPENDED" => Self::Suspended,
            "BANNED" => Self::Banned,
            "REVOKED" => Self::Revoked,
            "INACTIVE" => Self::Inactive,
            _ => Self::Unknown,
        }
    }

    /// Render `● LABEL`, optionally with the appropriate ANSI color.
    pub fn pill(self, label: &str, use_color: bool) -> String {
        if !use_color {
            return format!("● {label}");
        }
        match self {
            Self::Active => format!("{} {label}", "●".green()),
            Self::Expiring => format!("{} {label}", "●".yellow()),
            Self::Expired | Self::Suspended => format!("{} {label}", "●".bright_yellow()),
            Self::Banned | Self::Revoked => format!("{} {label}", "●".red()),
            Self::Inactive | Self::Unknown => format!("{} {label}", "●".bright_black()),
        }
    }
}
