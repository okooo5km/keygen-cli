use crate::{cli::Context, error::Result};

pub async fn run(_ctx: &Context) -> Result<()> {
    Err(crate::Error::user(
        "doctor not yet implemented (step 4 placeholder)",
    ))
}
