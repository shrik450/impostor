use crate::entry::EntryCompilationError;

#[derive(Debug)]
pub enum Error {
    Unknown,
    ParseError(inko_core::parser::Error),
    EntryCompilationError(EntryCompilationError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Unknown => write!(f, "unknown error"),
            Error::ParseError(e) => write!(f, "parse error: {}", e),
            Error::EntryCompilationError(e) => write!(f, "entry compilation error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
