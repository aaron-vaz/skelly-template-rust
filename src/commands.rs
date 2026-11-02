//! What each command does. They write to a [`Write`] instead of stdout so a test can read the output.

use std::io::Write;

use crate::error::Error;

/// Prints a greeting; an example command, replace it with your own.
pub(crate) fn greet(name: &str, out: &mut impl Write) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::EmptyName);
    }
    writeln!(out, "Hello, {name}!")?;
    Ok(())
}

/// Prints the name and version of the crate.
pub(crate) fn version(out: &mut impl Write) -> Result<(), Error> {
    writeln!(
        out,
        "{} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_prints_the_greeting() {
        // Given a name and somewhere to write
        let mut out = Vec::new();

        // When the command runs
        greet("Ferris", &mut out).unwrap();

        // Then the greeting is written
        assert_eq!(String::from_utf8(out).unwrap(), "Hello, Ferris!\n");
    }

    #[test]
    fn greet_rejects_a_blank_name() {
        // Given a blank name
        let mut out = Vec::new();

        // When the command runs
        let result = greet("  ", &mut out);

        // Then it fails and writes nothing
        assert!(matches!(result, Err(Error::EmptyName)));
        assert!(out.is_empty());
    }

    #[test]
    fn version_prints_name_and_version() {
        // Given somewhere to write
        let mut out = Vec::new();

        // When the command runs
        version(&mut out).unwrap();

        // Then the crate name and version are written
        let expected = format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }
}
