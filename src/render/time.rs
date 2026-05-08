use jiff::Timestamp;

/// Render an absolute timestamp as a relative phrase ("3 days ago" / "in 2 months").
pub fn relative(ts: Timestamp) -> String {
    let now = Timestamp::now();
    let span = ts - now;
    let secs = span.total(jiff::Unit::Second).unwrap_or(0.0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let abs = secs.abs().round() as u64;
    let phrase = humantime::format_duration(std::time::Duration::from_secs(abs)).to_string();
    if secs < 0.0 {
        format!("{phrase} ago")
    } else {
        format!("in {phrase}")
    }
}
