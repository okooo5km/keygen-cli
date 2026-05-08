pub mod app;
pub mod theme;
pub mod views;
pub mod widgets;

use crate::{cli::Context, error::Result};

pub async fn launch(_ctx: &Context) -> Result<()> {
    Err(crate::Error::user(
        "tui not yet implemented (step 14 placeholder)",
    ))
}
