//! Logging.

use tracing::Level;

/// Starts logging JSON lines to stdout at `level` and above.
///
/// # Panics
///
/// Panics if a global subscriber is already installed, which can only be a
/// programming error in `main`.
pub fn init_tracing(level: Level) {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(level)
        .init();
}
