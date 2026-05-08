use std::process::ExitCode;

use clap::Parser;
use keygen_cli::{cli::Cli, exit::ExitKind, run};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.globals.verbose);

    match run(cli).await {
        Ok(()) => ExitKind::Ok.into(),
        Err(err) => {
            let kind = ExitKind::from_error(&err);
            keygen_cli::output::report_error(&err);
            kind.into()
        }
    }
}

fn init_tracing(verbose: u8) {
    use tracing_subscriber::{fmt, EnvFilter};

    let default = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_env("KEYGEN_LOG").unwrap_or_else(|_| EnvFilter::new(default));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .compact()
        .init();
}
