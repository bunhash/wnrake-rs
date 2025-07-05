//! Authentication methods
//!
//! Used for both gluetun API endpoint and basic proxy authentication
//!
//! [https://github.com/qdm12/gluetun](https://github.com/qdm12/gluetun)

use crate::error::Error;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};

#[derive(Clone, Debug)]
pub enum Credentials {
    BasicAuth(BasicAuth),
    ApiKey(String),
}

impl Credentials {
    /// Creates BasicAuth credentials
    pub fn basic(username: &str, password: &str) -> Self {
        Credentials::BasicAuth(BasicAuth::new(username, password))
    }

    /// Creates ApiKey credentials
    pub fn api_key(api_key: &str) -> Self {
        Credentials::ApiKey(api_key.into())
    }

    /// Returns a header map to be included in the request.
    pub fn to_header(&self) -> Result<HeaderMap, Error> {
        let mut header = HeaderMap::new();
        match self {
            Credentials::BasicAuth(cred) => {
                header.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&cred.to_string()).map_err(Error::solver)?,
                );
            }
            Credentials::ApiKey(cred) => {
                header.insert(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(&cred).map_err(Error::solver)?,
                );
            }
        }
        Ok(header)
    }
}

#[derive(Clone, Debug)]
pub struct BasicAuth {
    username: String,
    password: String,
}

impl BasicAuth {
    /// New BasicAuth username-password pair
    pub fn new(username: &str, password: &str) -> Self {
        BasicAuth {
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    /// Encodes the header value. For gluetun, use `to_string()`.
    pub fn to_header_value(&self, encoder: fn(&str) -> &[u8]) -> String {
        let concat = format!("{}:{}", self.username, self.password);
        let bytes = encoder(&concat);
        format!("Basic {}", STANDARD.encode(bytes))
    }
}

impl ToString for BasicAuth {
    fn to_string(&self) -> String {
        self.to_header_value(|s| s.as_bytes())
    }
}
