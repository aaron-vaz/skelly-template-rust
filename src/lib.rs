{{- if eq .Inputs.project_type.Value "cli" -}}
//! The logic of `{{.Inputs.project_name.Value}}`. The binary in `main.rs` only wires it to the
//! process, so everything here can be tested without starting one.
//!
//! Modules are private and the few things other code needs are re-exported
//! below; widen `pub(crate)` to `pub` only when something outside needs it.

mod cli;
mod commands;
mod error;

use std::io::Write;

pub use cli::Cli;
pub use error::Error;

/// Runs the command chosen on the command line, writing what it prints to `out`.
///
/// # Errors
///
/// Returns an [`Error`] when the command rejects its input or `out` cannot be
/// written to.
pub fn run(cli: &Cli, out: &mut impl Write) -> Result<(), Error> {
    match &cli.command {
        cli::Command::Greet { name } => commands::greet(name, out),
        cli::Command::Version => commands::version(out),
    }
}
{{- else -}}
//! The logic of `{{.Inputs.project_name.Value}}`, a web service. The binary in `main.rs` only
//! reads the environment, starts logging and calls into this crate, so the
//! service can be tested without starting a process.
//!
//! Modules are private and the few things other code needs are re-exported
//! below; widen `pub(crate)` to `pub` only when something outside needs it.

mod config;
mod server;
mod telemetry;

pub use config::{Config, ConfigError};
pub use server::{router, serve, shutdown_signal};
pub use telemetry::init_tracing;
{{- end}}
