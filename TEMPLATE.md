# skelly-template-rust

A [skelly](https://github.com/aaron-vaz/skelly) template for Rust projects. One template, two kinds of project, picked with the `project_type` input:

| `project_type` | You get |
|---------------|---------|
| `cli`         | A command line app: a library with the testable logic (a `clap` derive definition, `greet` and `version` commands, an error type with `thiserror`) and a thin binary, plus a release workflow that attaches binaries and `checksums.txt` to the GitHub release |
| `service`     | A web service on `tokio` and `axum`: `/healthz`, `/version` and an example `/hello/{name}`, request logs with `tracing` as JSON, env-based config, graceful shutdown on SIGTERM and Ctrl+C, a Dockerfile and a workflow that publishes the image to GHCR |

Both are edition 2024 with a single package: `src/lib.rs` holds the logic, `src/main.rs` is wiring, modules are files and items are `pub(crate)` by default. Errors are `thiserror` in the library and `anyhow` only in `main`. Both come with a `Makefile`, a pinned toolchain, `[lints]` (clippy pedantic, `unwrap_used`/`expect_used` warned, `unsafe` forbidden), CI with fmt, clippy and tests, release-please, Dependabot, an `AGENTS.md` describing the conventions, and tests (unit tests next to the code, integration tests in `tests/`).

## Usage

```bash
skelly sources add rust github.com/aaron-vaz/skelly-template-rust
skelly init rust -dst ./my-app
```

Inputs are asked in alphabetical order: `owner` (default `aaron-vaz`), `project_name` (required), `project_type` (`cli` or `service`, default `cli`) and `rust_version` (default `1.97.0`).

`project_name` becomes the crate name and the binary name. skelly cannot check it, so use a valid crate name: letters, digits, `-` and `_`, starting with a letter. In code a `-` is written as `_` (`my-app` is `my_app`).

## How it works

`.skelly.yml` uses three skelly features to produce both variants from one tree:

- `choices` on `project_type`, so a typo is rejected instead of producing a half-empty project.
- `exclude` entries, each removing a path when `project_type` is not the one that needs it. `Cargo.toml`, `lib.rs`, `main.rs`, the `Makefile` and the docs are small enough to switch with `{{if}}` inside the file.
- `rename`, which moves `github/` to `.github` when the project is generated. The workflows and `dependabot.yml` for generated projects live in `github/`, so GitHub does not run them, and Dependabot does not read them, in this repository. The real `.github/` here holds only the template's own CI.

`github` is listed under `raw` because workflows use `${{ }}`. `raw` and `exclude` use the template's own names, which is why they say `github/...`. `TEMPLATE.md` and the real `.github` are excluded unconditionally: they belong to this repository, not to generated projects. Rust code must not contain `{{` (for example in a `format!`), or skelly reads it as a template action.

When you edit a workflow for generated projects, edit it under `github/workflows/`. It is not run here, so check changes with the test below.

## The toolchain pin

`rust_version` is written to `rust-toolchain.toml` (`channel`), to `rust-version` in `Cargo.toml` and to the Dockerfile's base image. It defaults to the version in the author's other Rust projects. The pin is deliberate: with `clippy::pedantic` and `-D warnings`, a new Rust release can turn a green tree red with no code changes. Bump the default in `.skelly.yml` deliberately, as its own commit, together with CI, which renders with the default. Generated CI installs whatever `rust-toolchain.toml` says (`make deps`), so the version is written once per project.

## Dependencies

Versions in `Cargo.toml` were picked with `cargo add` and are plain caret requirements; Dependabot keeps them current in generated projects. To refresh them here, render a variant, run `cargo upgrade` or `cargo add` there and copy the versions back. `Cargo.lock` is not part of the template: the first build in a generated project creates it, and the project commits it.

## Using it from a script or an AI agent

```bash
# What does the template ask for? JSON with defaults, choices and which inputs are required
skelly inspect rust

# Create a project without any prompt
skelly init rust -dst ./my-app -no-input -set project_name=my-app -set project_type=service
```

## Testing the template

`.github/workflows/ci.yml` renders each variant with skelly, checks for leftover template markers, runs `make all` on the result and smoke tests the service (start, `/healthz`, SIGTERM). To do the same locally:

```bash
skelly init . -dst /tmp/example-app -no-input -set project_name=example-app -set project_type=cli
cd /tmp/example-app && make deps all
```
