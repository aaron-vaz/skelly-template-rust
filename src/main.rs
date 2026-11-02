{{- if eq .Inputs.project_type.Value "cli" -}}
//! The `{{.Inputs.project_name.Value}}` binary: wires the library to the process and nothing else.

use std::io;
use std::process::ExitCode;

use clap::Parser;

use {{.Inputs.project_name.Value | replace "-" "_"}}::{Cli, run};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut stdout = io::stdout().lock();
    match run(&cli, &mut stdout) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // anyhow is for binaries: it turns the error into one line with its causes
            eprintln!("Error: {:#}", anyhow::Error::from(err));
            ExitCode::FAILURE
        }
    }
}
{{- else -}}
//! The `{{.Inputs.project_name.Value}}` binary: reads the environment, starts logging and serves.

use std::process::ExitCode;

use anyhow::Context;
use tokio::net::TcpListener;

use {{.Inputs.project_name.Value | replace "-" "_"}}::{Config, init_tracing, router, serve, shutdown_signal};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<()> {
    // The only place the environment is read; everything else gets a Config
    let config = Config::load(|key| std::env::var(key).ok()).context("invalid configuration")?;
    init_tracing(config.log_level);

    let listener = TcpListener::bind(config.addr)
        .await
        .with_context(|| format!("failed to listen on {}", config.addr))?;
    tracing::info!(addr = %config.addr, "server listening");

    serve(
        listener,
        router(),
        shutdown_signal(),
        config.shutdown_timeout,
    )
    .await
    .context("server failed")?;
    tracing::info!("server stopped");
    Ok(())
}
{{- end}}
