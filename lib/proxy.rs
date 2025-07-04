//! Proxy configuration
//!
//! Written to support gluetun VPNs. See:
//!
//! [https://github.com/qdm12/gluetun](https://github.com/qdm12/gluetun)

use crate::error::Error;
use serde_json::{Map, Value};
use std::fmt;

mod api;
mod auth;

pub use api::Api;
pub use auth::{BasicAuth, Credentials};

#[derive(Clone, Copy, Debug)]
pub enum ProxyStatus {
    Running,
    Stopped,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Proxy {
    pub(crate) url: String,
    pub(crate) credentials: Option<Credentials>,
    pub(crate) api: Option<Api>,
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Proxy {
    /// Creates a new proxy configuration
    pub fn new(url: &str) -> Self {
        Proxy {
            url: url.into(),
            credentials: None,
            api: None,
        }
    }

    /// Creates a new proxy configuration with control server
    pub fn with_api(url: &str, api: Api) -> Self {
        Proxy {
            url: url.into(),
            credentials: None,
            api: Some(api),
        }
    }

    /// Returns a reference to the URL
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    /// Returns a reference to the control server
    pub fn credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// Returns a reference to the control server
    pub fn api(&self) -> Option<&Api> {
        self.api.as_ref()
    }

    /// Returns the public IP, if possible
    pub async fn ip(&self) -> Option<String> {
        match &self.api {
            Some(api) => api.ip().await.ok(),
            None => None,
        }
    }

    /// Gets the status of the proxy
    pub async fn status(&self) -> Result<ProxyStatus, Error> {
        match &self.api {
            Some(api) => api.status().await,
            None => Ok(ProxyStatus::Unknown),
        }
    }

    /// Restarts the proxy. Can timeout.
    pub async fn restart(&self) -> Result<(), Error> {
        match &self.api {
            Some(api) => api.restart().await,
            None => Ok(()),
        }
    }

    /// Returns the proxy as a JSON serializable object
    pub(crate) fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("url".into(), Value::String(self.url.clone()));
        if let Some(Credentials::BasicAuth(creds)) = &self.credentials {
            map.insert("username".into(), Value::String(creds.username().into()));
            map.insert("password".into(), Value::String(creds.password().into()));
        }
        Value::Object(map)
    }
}

pub struct ProxyBuilder {
    url: String,
    credentials: Option<Credentials>,
    api: Option<Api>,
}

impl ProxyBuilder {
    /// Creates a new `ProxyBuilder` with the given proxy URL
    pub fn new(url: &str) -> Self {
        ProxyBuilder {
            url: url.into(),
            credentials: None,
            api: None,
        }
    }

    /// Adds basic auth credentials to the proxy
    pub fn credentials(mut self, username: &str, password: &str) -> Self {
        self.credentials = Some(Credentials::basic(username, password));
        self
    }

    /// Adds API functionality to the proxy (gluetun)
    pub fn api(mut self, api: Api) -> Self {
        self.api = Some(api);
        self
    }

    /// Consumes the builder and returns the proxy
    pub fn build(self) -> Proxy {
        Proxy {
            url: self.url,
            credentials: self.credentials,
            api: self.api,
        }
    }
}
