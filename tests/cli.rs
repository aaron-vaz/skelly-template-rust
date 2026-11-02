//! End-to-end tests: run the compiled binary as a sub-process.

// clippy.toml only exempts #[test] functions, and these files' helpers are tests in spirit:
// a panic in them is the failure report.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::{Command, Output};

use clap::Parser;

use {{.Inputs.project_name.Value | replace "-" "_"}}::{Cli, run};

fn binary(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_{{.Inputs.project_name.Value}}"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn greet_prints_the_greeting() {
    // Given the compiled binary
    // When it is asked to greet someone
    let output = binary(&["greet", "--name", "Ferris"]);

    // Then it succeeds and prints the greeting on stdout
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, Ferris!\n");
}

#[test]
fn version_prints_the_crate_version() {
    // Given the compiled binary
    // When it is asked for its version
    let output = binary(&["version"]);

    // Then it prints the package name and version
    assert!(output.status.success());
    let expected = format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn a_blank_name_fails_on_stderr() {
    // Given the compiled binary
    // When it is asked to greet nobody
    let output = binary(&["greet", "--name", " "]);

    // Then it fails, prints nothing on stdout and explains on stderr
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Error: name must not be empty\n"
    );
}

#[test]
fn an_unknown_command_is_a_usage_error() {
    // Given the compiled binary
    // When it is given a command it does not have
    let output = binary(&["nope"]);

    // Then clap rejects it with its usage exit code
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn run_works_through_the_library() {
    // Given a parsed command line
    let cli = Cli::try_parse_from(["app", "greet"]).unwrap();
    let mut out = Vec::new();

    // When it is run without a process
    run(&cli, &mut out).unwrap();

    // Then the default greeting is written
    assert_eq!(String::from_utf8(out).unwrap(), "Hello, World!\n");
}
