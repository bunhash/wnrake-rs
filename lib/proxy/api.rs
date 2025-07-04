//! Primitive gluetun API

use crate::{
    error::Error,
    proxy::{Credentials, ProxyStatus},
};
use reqwest::Client;
use serde_json::{Map, Value};
use std::{fmt, time::Duration};
use tokio::time::timeout;

#[derive(Clone, Debug)]
pub struct Api {
    url: String,
    credentials: Option<Credentials>,
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Api({})", self.url)
    }
}

impl Api {
    /// Creates a new proxy configuration
    pub fn new(url: &str) -> Self {
        Api {
            url: url.into(),
            credentials: None,
        }
    }

    /// Creates a new proxy configuration with control server
    pub fn with_basic_auth(url: &str, username: &str, password: &str) -> Self {
        Api {
            url: url.into(),
            credentials: Some(Credentials::basic(username, password)),
        }
    }

    /// Creates a new proxy configuration with control server
    pub fn with_api_key(url: &str, api_key: &str) -> Self {
        Api {
            url: url.into(),
            credentials: Some(Credentials::api_key(api_key)),
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

    /// Gets the status of the proxy
    pub async fn ip(&self) -> Result<String, Error> {
        let url = format!("{}/v1/publicip/ip", &self.url);
        let mut client = Client::new().get(url);
        if let Some(cred) = &self.credentials {
            client = client.headers(cred.to_header()?);
        }
        let res = client.send().await?;
        let res = res.json::<Value>().await.map_err(Error::json)?;
        Ok(res["public_ip"]
            .as_str()
            .ok_or(Error::json("expected public_ip in response"))?
            .into())
    }

    /// Gets the status of the proxy
    pub async fn status(&self) -> Result<ProxyStatus, Error> {
        let url = format!("{}/v1/openvpn/status", &self.url);
        let mut client = Client::new().get(url);
        if let Some(cred) = &self.credentials {
            client = client.headers(cred.to_header()?);
        }
        let res = client.send().await?;
        let res = res.json::<Value>().await.map_err(Error::json)?;
        Ok(match res["status"].as_str() {
            Some("running") => ProxyStatus::Running,
            Some("stopped") => ProxyStatus::Stopped,
            _ => ProxyStatus::Unknown,
        })
    }

    /// Restarts the proxy. Can timeout.
    pub async fn restart(&self) -> Result<(), Error> {
        let _ = self.put_state("stopped").await;
        std::thread::sleep(Duration::from_millis(1000));
        let _ = self.put_state("running").await;
        match timeout(Duration::from_secs(60), self.wait_for_proxy()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::timeout("waiting for proxy timed out")),
        }
    }

    /// Wait for proxy to become good. Infinite loop.
    async fn wait_for_proxy(&self) {
        loop {
            match self.status().await {
                Ok(ProxyStatus::Running) => break,
                _ => std::thread::sleep(Duration::from_millis(500)),
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    /// Send a command the modify the state
    async fn put_state(&self, state: &str) -> Result<(), Error> {
        let url = format!("{}/v1/openvpn/status", &self.url);
        let mut map = Map::new();
        map.insert("status".into(), state.into());

        // Build and send PUT
        let mut client = Client::new().put(url);
        if let Some(cred) = &self.credentials {
            client = client.headers(cred.to_header()?);
        }
        client.json(&map).send().await?;
        Ok(())
    }
}
