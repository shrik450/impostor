use std::fmt::Debug;

use crate::entry::EntryCompilationError;

#[derive(Debug)]
pub enum Error {
    Unknown,
    ParseError(impostor_core::parser::Error),
    EntryCompilationError(EntryCompilationError),
    InvalidMethod(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Unknown => write!(f, "unknown error"),
            Error::ParseError(e) => write!(f, "parse error: {}", e),
            Error::EntryCompilationError(e) => write!(f, "entry compilation error: {}", e),
            Error::InvalidMethod(e) => write!(f, "invalid method: {}", e),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
