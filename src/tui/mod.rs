pub mod app;
pub mod permission;
pub mod state;
pub mod theme;
pub mod views;
pub mod widgets;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{cli::Context, error::Result};

/// Launch the full-screen ratatui dashboard.
pub async fn launch(ctx: &Context) -> Result<()> {
    enable_raw_mode().map_err(|e| crate::Error::user(format!("tui: enable raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| crate::Error::user(format!("tui: enter alt screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| crate::Error::user(format!("tui: terminal: {e}")))?;

    let result = app::run(&mut terminal, ctx).await;

    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}
