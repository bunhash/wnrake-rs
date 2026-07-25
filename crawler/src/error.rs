//! Errors

use std::fmt;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum ErrorType {
    /// Config errors
    Config,

    /// IO errors
    Io,

    /// JSON parsing errors
    Json,

    /// Proxy errors
    Proxy,

    /// Errors solving the solution
    Solution,

    /// Network errors between the client and flaresolverr
    Solver,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Config => f.write_str("config"),
            ErrorType::Io => f.write_str("io"),
            ErrorType::Json => f.write_str("json"),
            ErrorType::Proxy => f.write_str("proxy"),
            ErrorType::Solution => f.write_str("solution"),
            ErrorType::Solver => f.write_str("solver"),
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

impl Error {
    pub fn parse_solution_error(msg: impl fmt::Display) -> Error {
        let message = format!("{}", msg);
        if message.contains("ERR_TUNNEL_CONNECTION_FAILED") {
            Error {
                error_type: ErrorType::Proxy,
                fatal: true,
                message,
            }
        } else if message.contains("Error solving the challenge") {
            Error {
                error_type: ErrorType::Solution,
                fatal: false,
                message,
            }
        } else {
            Error {
                error_type: ErrorType::Solver,
                fatal: true,
                message: format!("{}", msg),
            }
        }
    }

    pub fn config(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Config,
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

    pub fn json(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Json,
            fatal: true,
            message: format!("{}", msg),
        }
    }

    pub fn proxy(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Proxy,
            fatal: false,
            message: format!("{}", msg),
        }
    }

    pub fn solution(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Solution,
            fatal: false,
            message: format!("{}", msg),
        }
    }

    pub fn solver(msg: impl fmt::Display) -> Error {
        Error {
            error_type: ErrorType::Solver,
            fatal: true,
            message: format!("{}", msg),
        }
    }
}
