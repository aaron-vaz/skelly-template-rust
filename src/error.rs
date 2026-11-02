//! The errors of the library.

use std::io;

use thiserror::Error;

/// Why a command could not be run.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The name to greet was empty.
    #[error("name must not be empty")]
    EmptyName,
    /// The output could not be written.
    #[error("failed to write output")]
    Io(#[from] io::Error),
}
