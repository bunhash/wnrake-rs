//! config parameters

use config::{File, FileFormat};
use serde::Deserialize;
use std::collections::HashMap;
use wnrake::{
    cache::Cache,
    client::Client,
    proxy::{Api, Proxy},
};

#[derive(Clone, Debug)]
pub struct Config {
    /// Configuration file
    config_file: Option<String>,

    /// Solver URL [default: http://localhost:8191/v1]
    solver: String,

    /// Cache [default: disabled]
    cache: Option<String>,

    /// Cache [default: disabled]
    proxy: Option<ProxyConfig>,
}

impl Config {
    /// Merges CLI and file configurations. CLI takes precendence. Comsumes everything.
    pub fn merge(
        config_file: Option<String>,
        solver: Option<String>,
        disable_cache: bool,
        cache: Option<String>,
        proxy_name: Option<String>,
        mut file: ConfigFile,
    ) -> Config {
        let solver = solver.unwrap_or(file.solver);
        let cache = if disable_cache {
            None
        } else if cache.is_some() {
            cache
        } else {
            file.cache
        };
        let proxy_name = proxy_name.or(file.proxy);
        let proxy = match proxy_name {
            Some(name) => match file.proxies.remove(&name) {
                Some(proxy) => Some(proxy),
                None => {
                    log::warn!("`{}` proxy does not exist in configuration", name);
                    None
                }
            },
            None => None,
        };
        Config {
            config_file,
            solver,
            cache,
            proxy,
        }
    }

    /// Builds a `Client` given the configuration
    pub fn to_client(&self) -> Client {
        let mut client = Client::builder(&self.solver);
        if let Some(proxy) = self.configure_proxy() {
            client = client.proxy(proxy);
        }
        if let Some(cache) = self.configure_cache() {
            client = client.cache(cache);
        }
        client.build()
    }

    fn configure_proxy(&self) -> Option<Proxy> {
        Some(self.proxy.as_ref()?.to_proxy())
    }

    fn configure_cache(&self) -> Option<Cache> {
        Some(Cache::new(self.cache.as_ref()?).ok()?)
    }

    pub fn load_config_file(&self) -> ConfigFile {
        match &self.config_file {
            Some(file) => ConfigFile::load(file),
            None => ConfigFile::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigFile {
    /// Solver URL [default: http://localhost:8191/v1]
    #[serde(default = "solver_default")]
    pub solver: String,

    /// Cache [default: disabled]
    pub cache: Option<String>,

    /// Proxy name [default: disabled]
    pub proxy: Option<String>,

    /// Map of proxies
    #[serde(default)]
    pub proxies: HashMap<String, ProxyConfig>,
}

fn solver_default() -> String {
    "http://localhost:8191/v1".into()
}

impl Default for ConfigFile {
    fn default() -> Self {
        ConfigFile {
            solver: solver_default(),
            cache: None,
            proxy: None,
            proxies: HashMap::new(),
        }
    }
}

impl ConfigFile {
    /// Loads the configuration file (TOML)
    ///
    /// Example:
    ///
    /// solver = "http://localhost:8191/v1"
    /// cache = "/path/to/cache_dir"
    ///
    /// [proxies]
    /// proxy1 = { url = "http://localhost:9000" }
    /// proxy2 = { url = "http://localhost:9000", api = "http://localhost:8000" }
    pub fn load(file: &str) -> Self {
        log::debug!("Using configuration {}", file);
        match ::config::Config::builder()
            .add_source(File::new(file, FileFormat::Toml))
            .build()
        {
            Ok(config_file) => match config_file.try_deserialize::<ConfigFile>() {
                Ok(config) => return config,
                Err(e) => log::warn!("{:?}", e),
            },
            Err(e) => log::warn!("{:?}", e),
        }
        ConfigFile::default()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Proxy username
    pub username: Option<String>,

    /// Proxy password
    pub password: Option<String>,

    /// Proxy API
    pub api: Option<String>,

    /// Proxy API username
    pub api_username: Option<String>,

    /// Proxy API password
    pub api_password: Option<String>,

    /// Proxy API key
    pub api_key: Option<String>,
}

impl ProxyConfig {
    pub fn to_proxy(&self) -> Proxy {
        // Build API
        let api = match self.api.as_deref() {
            Some(url) => {
                if let Some(key) = self.api_key.as_deref() {
                    Some(Api::with_api_key(url, key))
                } else if let (Some(username), Some(password)) =
                    (self.api_username.as_deref(), self.api_password.as_deref())
                {
                    Some(Api::with_basic_auth(url, username, password))
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
        proxy.build()
    }
}
