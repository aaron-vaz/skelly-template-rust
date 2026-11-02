# AGENTS.md

Guidelines for agentic coding agents working in this repository.

## Project Overview

`{{.Inputs.project_name.Value}}` is a Rust {{if eq .Inputs.project_type.Value "cli"}}command line application{{else}}web service{{end}}.

- **Language**: Rust, edition 2024
- **Toolchain**: {{.Inputs.rust_version.Value}}, pinned in `rust-toolchain.toml` (also `rust-version` in `Cargo.toml`)
- **Crate and binary**: `{{.Inputs.project_name.Value}}` (`github.com/{{.Inputs.owner.Value}}/{{.Inputs.project_name.Value}}`)

## Build/Lint/Test Commands

```bash
make all        # Runs lint, test, and build
make build      # Optimized build -> target/release/{{.Inputs.project_name.Value}}
make test       # Run all tests
make lint       # cargo fmt --check, then clippy --all-targets -D warnings
make fmt        # Format code
make deps       # Install the pinned toolchain and fetch crates
{{- if eq .Inputs.project_type.Value "cli"}}
make release    # Test, then build/bin/{{.Inputs.project_name.Value}}_<os>_<arch>
{{- else}}
make run        # Run the service locally
make docker     # Build the container image
{{- end}}
cargo test name # Run the tests whose name contains `name`
```

## Code Style Guidelines

### Layout

A single package: the logic is in a library so it can be tested without a process, and the binary is thin.

```
src/lib.rs          # Declares the modules, re-exports what the binary and tests need
src/main.rs         # Wiring only: arguments or environment in, exit code out
{{- if eq .Inputs.project_type.Value "cli"}}
src/cli.rs          # Command line definition (clap derive)
src/commands.rs     # What each command does
src/error.rs        # The library's error type
tests/cli.rs        # Runs the compiled binary, and the library without a process
{{- else}}
src/config.rs       # Configuration from the environment, read once
src/server.rs       # Routes, serve with graceful shutdown, shutdown signal
src/telemetry.rs    # JSON logging
tests/server.rs     # A real server on a free port
{{- end}}
```

Modules are files, declared privately in `lib.rs`. Items are `pub(crate)` by default; make one `pub` (and re-export it from `lib.rs`) only when the binary or an integration test needs it.

### Lints

`Cargo.toml` sets `clippy::pedantic` to warn, warns on `unwrap_used` and `expect_used`, and forbids `unsafe_code`. CI runs clippy with `-D warnings`, so every warning fails the build. `clippy.toml` allows `unwrap`/`expect` inside tests, where a panic is the failure report. Fix a warning rather than allowing it; if an `#[allow]` is truly needed, say why in a comment.

### Toolchain and dependencies

The toolchain is pinned because a new Rust release can add lints and turn a green tree red. Bump it deliberately, as its own commit, changing `rust-toolchain.toml` and `rust-version` in `Cargo.toml` together (CI and the Dockerfile read it from there). Add dependencies with `cargo add`, which picks the latest compatible version, and commit `Cargo.lock`.

### Imports

Group `use` lines with blank lines between: standard library, third-party crates, then this crate.

### Naming Conventions

- Types and traits in `UpperCamelCase`; functions, modules and variables in `snake_case`; constants in `UPPER_SNAKE_CASE`
- Keep acronyms as the standard library does (`Io`, not `IO`, in type names)

### Error Handling

- The library returns its own error types, derived with `thiserror`; never panic in library code
- `anyhow` is for `main` only, where it adds context and prints the chain
- Error messages are one lowercase line without a trailing period; wrap sources with `#[from]` or `#[source]`
- Document `# Errors` on every public function that returns a `Result`
- `main` is the only place that prints errors and chooses the exit code

### Documentation

- Document all public items; start each module with a `//!` summary
- Comment "why", not "what"

### Testing Conventions

- Unit tests live in a `#[cfg(test)] mod tests` at the bottom of the file they test
- Name tests for the behaviour: `parse_rejects_bad_command_lines`
- Use table-driven tests (a loop over cases) for several inputs
- **BDD Syntax**: Structure tests using BDD-style comments (`// Given`, `// When`, `// Then`, `// And`)
{{- if eq .Inputs.project_type.Value "cli"}}
- **Integration Tests**: `tests/cli.rs` runs the compiled binary through `CARGO_BIN_EXE_{{.Inputs.project_name.Value}}`
{{- else}}
- **Integration Tests**: `tests/server.rs` starts the server on port 0 and talks to it over TCP; every wait is bounded with `tokio::time::timeout`
- Test handlers without a socket by sending requests to `router()` with `tower::ServiceExt::oneshot`
{{- end}}
{{- if eq .Inputs.project_type.Value "service"}}

### Service

- Everything is configured through environment variables, read once in `Config::load` (called from `main`); never call `std::env::var` elsewhere
- Log with `tracing`; logs are JSON on stdout. Never log secrets
- Shutdown on SIGTERM or Ctrl+C is graceful and bounded by `SHUTDOWN_TIMEOUT`; keep that when adding endpoints
{{- end}}

### Code Formatting

- Run `make fmt` and `make lint` before committing
- No unused imports or variables; the compiler and clippy enforce it
