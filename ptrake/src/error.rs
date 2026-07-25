//! Errors

use std::fmt;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum ErrorType {
    /// Crawler errors
    Crawler,

    /// Html errors
    Html,

    /// IO errors
    Io,

    /// Login errors
    Login,

    /// Reqwest errors
    Reqwest,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Crawler => f.write_str("crawler"),
            ErrorType::Html => f.write_str("html"),
            ErrorType::Io => f.write_str("io"),
            ErrorType::Login => f.write_str("login"),
            ErrorType::Reqwest => f.write_str("reqwest"),
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

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        let mut err: &dyn std::error::Error = &error;
        while let Some(source) = err.source() {
            err = source;
        }
        Error::reqwest(err)
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

    pub fn html(msg: impl fmt::Display, fatal: bool) -> Error {
        Error {
            error_type: ErrorType::Html,
            fatal,
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

    pub fn login(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Login,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn reqwest(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Reqwest,
            fatal: true,
            message: format!("{}", msg),
        }
    }
}
