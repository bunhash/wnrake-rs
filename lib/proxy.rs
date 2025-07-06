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

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum ProxyStatus {
    Running,
    Stopped,
    Unknown,
}

impl fmt::Display for ProxyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyStatus::Running => f.write_str("running"),
            ProxyStatus::Stopped => f.write_str("stopped"),
            ProxyStatus::Unknown => f.write_str("unknown"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Proxy {
    pub(crate) url: String,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) api: Option<Api>,
}

impl Proxy {
    /// Creates a new proxy configuration
    pub fn new(url: &str) -> Self {
        Proxy {
            url: url.into(),
            username: None,
            password: None,
            api: None,
        }
    }

    /// Creates a new proxy configuration with control server
    pub fn with_api(url: &str, api: Api) -> Self {
        Proxy {
            url: url.into(),
            username: None,
            password: None,
            api: Some(api),
        }
    }

    /// Creates a proxy builder
    pub fn builder(url: &str) -> ProxyBuilder {
        ProxyBuilder::new(url)
    }

    /// Returns a reference to the URL
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    /// Returns a reference to the username
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns a reference to the password
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Returns a reference to the API URL
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

    /// Wait for the proxy to be good
    pub async fn wait(&self, seconds: u64) -> Result<(), Error> {
        match &self.api {
            Some(api) => api.wait_for_ip(seconds).await,
            None => Ok(()),
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
        if let Some(username) = self.username() {
            map.insert("username".into(), Value::String(username.into()));
            if let Some(password) = self.username() {
                map.insert("password".into(), Value::String(password.into()));
            }
        }
        Value::Object(map)
    }
}

pub struct ProxyBuilder {
    url: String,
    username: Option<String>,
    password: Option<String>,
    api: Option<Api>,
}

impl ProxyBuilder {
    /// Creates a new `ProxyBuilder` with the given proxy URL
    pub fn new(url: &str) -> Self {
        ProxyBuilder {
            url: url.into(),
            username: None,
            password: None,
            api: None,
        }
    }

    /// Adds basic auth credentials to the proxy
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Adds basic auth credentials to the proxy
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.into());
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
            username: self.username,
            password: self.password,
            api: self.api,
        }
    }
}
