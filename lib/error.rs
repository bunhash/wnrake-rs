//! Errors

use std::fmt;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum ErrorType {
    /// HTML parsing errors
    Html,

    /// IO errors
    Io,

    /// JSON parsing errors
    Json,

    /// Parser errors
    Parser,

    /// Errors with the solution (i.e. status != ok)
    Solution,

    /// Network errors between the client and flaresolverr
    Solver,

    /// When the solution contains a non-200 status
    Status,

    /// General timeouts (not flaresolver timeouts)
    Timeout,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Html => f.write_str("html"),
            ErrorType::Io => f.write_str("io"),
            ErrorType::Json => f.write_str("json"),
            ErrorType::Parser => f.write_str("parser"),
            ErrorType::Solution => f.write_str("solution"),
            ErrorType::Solver => f.write_str("solver"),
            ErrorType::Status => f.write_str("status"),
            ErrorType::Timeout => f.write_str("timeout"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::io(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        let mut err: &dyn std::error::Error = &error;
        while let Some(source) = err.source() {
            err = source;
        }
        Error::solver(err)
    }
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for Error {
    fn from(error: scraper::error::SelectorErrorKind<'a>) -> Self {
        Error::html(error)
    }
}

impl Error {
    pub fn html(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Html,
            message: format!("{}", msg),
        }
    }

    pub fn io(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Io,
            message: format!("{}", msg),
        }
    }

    pub fn json(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Json,
            message: format!("{}", msg),
        }
    }

    pub fn parser(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Parser,
            message: format!("{}", msg),
        }
    }

    pub fn solution(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Solution,
            message: format!("{}", msg),
        }
    }

    pub fn solver(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Solver,
            message: format!("{}", msg),
        }
    }

    pub fn status(status: u16) -> Error {
        Error {
            error_type: ErrorType::Status,
            message: format!("returned status {}", status),
        }
    }

    pub fn timeout(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Timeout,
            message: format!("{}", msg),
        }
    }
}
