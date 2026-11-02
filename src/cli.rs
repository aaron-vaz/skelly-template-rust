//! The command line definition, parsed with clap.

use clap::{Parser, Subcommand};

/// The command line of the program.
#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help = true)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// The things the program can do; add your own here.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Print a greeting
    Greet {
        /// Who to greet
        #[arg(long, default_value = "World")]
        name: String,
    },
    /// Print the version
    Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_greet_uses_the_name_flag() {
        // Given a command line with a name
        let args = ["app", "greet", "--name", "Ferris"];

        // When it is parsed
        let cli = Cli::try_parse_from(args).unwrap();

        // Then the greet command carries the name
        assert_eq!(
            cli.command,
            Command::Greet {
                name: "Ferris".to_owned()
            }
        );
    }

    #[test]
    fn parse_greet_defaults_the_name() {
        // Given a greet without a name
        let args = ["app", "greet"];

        // When it is parsed
        let cli = Cli::try_parse_from(args).unwrap();

        // Then the name is the default
        assert_eq!(
            cli.command,
            Command::Greet {
                name: "World".to_owned()
            }
        );
    }

    #[test]
    fn parse_rejects_bad_command_lines() {
        // Given command lines that cannot be run
        let cases = [
            vec!["app"],
            vec!["app", "nope"],
            vec!["app", "greet", "--nope"],
            vec!["app", "greet", "extra"],
        ];

        for args in cases {
            // When they are parsed
            let result = Cli::try_parse_from(args.clone());

            // Then they are rejected
            assert!(result.is_err(), "args: {args:?}");
        }
    }
}
