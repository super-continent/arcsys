use thiserror::Error;

/// The `arcsys` standard error type
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Rebuilder error: {0}")]
    Rebuilder(String),
    #[error("Pac file has no entries")]
    NoPacEntries,
}
