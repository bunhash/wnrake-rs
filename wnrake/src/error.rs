//! Errors

use std::fmt;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum ErrorType {
    /// Crawler errors
    Crawler,

    /// Epub errors
    Epub,

    /// Html errors
    Html,

    /// IO errors
    Io,

    /// Json errors
    Json,

    /// Parser errors
    Parser,

    /// Status errors
    Status,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Crawler => f.write_str("crawler"),
            ErrorType::Epub => f.write_str("epub"),
            ErrorType::Html => f.write_str("html"),
            ErrorType::Io => f.write_str("io"),
            ErrorType::Json => f.write_str("json"),
            ErrorType::Parser => f.write_str("parser"),
            ErrorType::Status => f.write_str("status"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub fatal: bool,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl From<crawler::Error> for Error {
    fn from(error: crawler::Error) -> Error {
        Error {
            error_type: ErrorType::Crawler,
            fatal: error.fatal,
            message: format!("{}", error),
        }
    }
}

impl<'a> From<epub_builder::Error> for Error {
    fn from(error: epub_builder::Error) -> Self {
        Error::epub(error)
    }
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for Error {
    fn from(error: scraper::error::SelectorErrorKind<'a>) -> Self {
        Error::html(error, true)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::io(error)
    }
}

impl Error {
    pub fn crawler(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Crawler,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn epub(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Epub,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn html(msg: impl fmt::Display, fatal: bool) -> Error {
        Error {
            error_type: ErrorType::Html,
            fatal,
            message: format!("{}", msg),
        }
    }

    pub fn json(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Json,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn io(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Io,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn parser(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Parser,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn status(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Status,
            fatal: true,
            message: format!("{}", msg),
        }
    }
}
