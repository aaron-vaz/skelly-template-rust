# {{.Inputs.project_name.Value}}

{{if eq .Inputs.project_type.Value "cli" -}}
A Rust command line application.
{{- else -}}
A Rust web service.
{{- end}}

## Development

Requires [rustup](https://rustup.rs) and a C toolchain (for linking). The Rust toolchain ({{.Inputs.rust_version.Value}}) is pinned in `rust-toolchain.toml`.

```bash
make deps      # install the pinned toolchain, with rustfmt and clippy, and fetch crates
make all       # lint, test and build
make test      # unit, integration and doc tests
make lint      # cargo fmt --check, then clippy with warnings as errors
make fmt       # format the code
make build     # optimized build -> target/release/{{.Inputs.project_name.Value}}
make run       # run it
{{- if eq .Inputs.project_type.Value "cli"}}
make release   # test, then copy the binary to build/bin/{{.Inputs.project_name.Value}}_<os>_<arch>
{{- else}}
make docker    # build the container image
{{- end}}
make help      # every target
```

Clippy runs with `pedantic` plus `unwrap_used` and `expect_used`, and `unsafe` code is forbidden (see `[lints]` in `Cargo.toml`). Commit `Cargo.lock`: the first build creates it.
{{- if eq .Inputs.project_type.Value "cli"}}

## Usage

```bash
{{.Inputs.project_name.Value}} --help
{{.Inputs.project_name.Value}} greet --name Ferris
{{.Inputs.project_name.Value}} version
```

To add a command, add a variant to `Command` in `src/cli.rs`, implement it in `src/commands.rs` and handle it in `run` in `src/lib.rs`.
{{- else}}

## Running

```bash
make run
curl localhost:8080/healthz
curl localhost:8080/hello/Ferris
```

The service is configured through environment variables, read once in `src/config.rs`:

| Variable           | Default        | Description                                                  |
|--------------------|----------------|--------------------------------------------------------------|
| `ADDR`             | `0.0.0.0:8080` | Address to listen on                                         |
| `LOG_LEVEL`        | `info`         | `error`, `warn`, `info`, `debug` or `trace`; logs are JSON   |
| `SHUTDOWN_TIMEOUT` | `10`           | Seconds in-flight requests get to finish on SIGTERM or Ctrl+C |

Endpoints live in `src/server.rs`.
{{- end}}

## Releases

Commits follow [Conventional Commits](https://www.conventionalcommits.org/). [release-please](https://github.com/googleapis/release-please) keeps a release pull request up to date on `main`, bumping `version` in `Cargo.toml`; merging it tags a release, which {{if eq .Inputs.project_type.Value "cli" -}}
builds the binary on Linux and macOS for amd64 and arm64 and attaches them to the GitHub release as `{{.Inputs.project_name.Value}}_<os>_<arch>`, with a `checksums.txt` that `sha256sum -c` reads.
{{- else -}}
builds the container image and pushes it to `ghcr.io/{{.Inputs.owner.Value}}/{{.Inputs.project_name.Value}}`.
{{- end}}

The release-please workflow needs a `RELEASE_PLEASE_PAT` repository secret (a personal access token) so that the tag it creates triggers the release workflow. Dependabot proposes updates for Cargo dependencies and GitHub Actions weekly.
