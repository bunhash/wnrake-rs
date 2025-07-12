//! Configuration File

use crate::{
    cache::Cache,
    client::Client,
    error::Error,
    proxy::{Api, Proxy},
};
use config::{File, FileFormat};
use serde::Deserialize;
use std::collections::{HashMap, hash_map::Keys};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Solver URL [default: http://localhost:8191/v1]
    #[serde(default = "solver_default")]
    solver: String,

    /// Cache [default: disabled]
    cache: Option<String>,

    /// Proxy name [default: disabled]
    proxy: Option<String>,

    /// Map of proxies
    #[serde(default)]
    proxies: HashMap<String, ProxyConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            solver: solver_default(),
            cache: None,
            proxy: None,
            proxies: HashMap::default(),
        }
    }
}

impl Config {
    pub fn builder(file: &str) -> Result<ConfigBuilder, Error> {
        ConfigBuilder::new(file)
    }

    /// Loads the configuration file (TOML)
    ///
    /// Example:
    ///
    /// solver = "http://localhost:8191/v1"
    /// cache = "/path/to/cache_dir"
    /// proxy = "proxy2"
    ///
    /// [proxies]
    /// proxy1 = { url = "http://localhost:9000" }
    /// proxy2 = { url = "http://localhost:9000", api = "http://localhost:8000" }
    /// proxy3 = { url = "http://vpn:8888", api = "http://vpn:8000", api_key = "ABCDEFGHIJKLMNOP" }
    pub fn load(file: &str) -> Result<Self, Error> {
        log::debug!("Using configuration {}", file);
        ::config::Config::builder()
            .add_source(File::new(file, FileFormat::Toml))
            .build()
            .map_err(Error::config)?
            .try_deserialize::<Config>()
            .map_err(Error::config)
    }

    /// Returns a reference to the solver URL
    pub fn solver(&self) -> &str {
        self.solver.as_str()
    }

    /// Returns a reference to the cache directory
    pub fn cache(&self) -> Option<&str> {
        self.cache.as_deref()
    }

    /// Returns a reference to the proxy name
    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    /// Returns an iterator to the proxy names
    pub fn proxies(&self) -> Keys<'_, String, ProxyConfig> {
        self.proxies.keys()
    }

    /// Builds a `Client`
    pub fn to_client(&self) -> Result<Client, Error> {
        self.build_client(self.proxy.as_deref())
    }

    /// Builds a `Client` with the provided proxy
    pub fn to_client_with_proxy(&self, proxy: &str) -> Result<Client, Error> {
        self.build_client(Some(proxy))
    }

    /// Builds a `Client` with the provided proxy
    fn build_client(&self, proxy: Option<&str>) -> Result<Client, Error> {
        let mut client = Client::builder(&self.solver);
        if let Some(proxy) = proxy {
            match self.proxies.get(proxy) {
                Some(pconf) => client = client.proxy(pconf.to_proxy()?),
                None => return Err(Error::config(format!("invalid proxy `{}`", proxy))),
            }
        }
        if let Some(cachedir) = self.cache.as_ref() {
            client = client.cache(Cache::new(cachedir)?);
        }
        Ok(client.build())
    }
}

#[derive(Clone, Debug)]
pub struct ConfigBuilder {
    inner: Config,
}

impl ConfigBuilder {
    pub fn default() -> Self {
        ConfigBuilder {
            inner: Config::default(),
        }
    }

    pub fn new(file: &str) -> Result<Self, Error> {
        Ok(ConfigBuilder {
            inner: Config::load(file)?,
        })
    }

    pub fn solver(mut self, solver: Option<String>) -> Self {
        if let Some(solver) = solver {
            self.inner.solver = solver;
        }
        self
    }

    pub fn disable_cache(mut self, disable_cache: bool) -> Self {
        if disable_cache {
            self.inner.cache = None;
        }
        self
    }

    pub fn cache(mut self, cache: Option<String>) -> Self {
        if let Some(cache) = cache {
            self.inner.cache = Some(cache);
        }
        self
    }

    pub fn proxy(mut self, proxy: Option<String>) -> Self {
        if let Some(proxy) = proxy {
            self.inner.proxy = Some(proxy);
        }
        self
    }

    pub fn build(self) -> Config {
        self.inner
    }
}

/// Default solver URL
fn solver_default() -> String {
    "http://localhost:8191/v1".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    url: String,

    /// Proxy username
    username: Option<String>,

    /// Proxy password
    password: Option<String>,

    /// Proxy API
    api: Option<String>,

    /// Proxy API username
    api_username: Option<String>,

    /// Proxy API password
    api_password: Option<String>,

    /// Proxy API key
    api_key: Option<String>,
}

impl ProxyConfig {
    /// Returns a reference to the proxy URL
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Returns a reference to the proxy username
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns a reference to the proxy password
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Returns a reference to the proxy API URL
    pub fn api(&self) -> Option<&str> {
        self.api.as_deref()
    }

    /// Returns a reference to the proxy API username
    pub fn api_username(&self) -> Option<&str> {
        self.api_username.as_deref()
    }

    /// Returns a reference to the proxy API password
    pub fn api_password(&self) -> Option<&str> {
        self.api_password.as_deref()
    }

    /// Returns a reference to the proxy API key
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Builds a `Proxy` given the configuration
    pub fn to_proxy(&self) -> Result<Proxy, Error> {
        // Build API
        let api = match self.api.as_deref() {
            Some(url) => {
                if let Some(key) = self.api_key.as_deref() {
                    Some(Api::with_api_key(url, key))
                } else if let Some(username) = self.api_username.as_deref() {
                    match self.api_password.as_deref() {
                        Some(password) => Some(Api::with_basic_auth(url, username, password)),
                        None => {
                            return Err(Error::config(
                                "API basic authentication requires both username and password",
                            ));
                        }
                    }
                } else {
                    Some(Api::new(url))
                }
            }
            None => None,
        };

        // Build Proxy
        let mut proxy = Proxy::builder(&self.url);
        if let Some(username) = self.username.as_deref() {
            proxy = proxy.username(username);
            if let Some(password) = self.password.as_deref() {
                proxy = proxy.password(password);
            }
        }
        if let Some(api) = api {
            proxy = proxy.api(api);
        }
        Ok(proxy.build())
    }
}
