//! config parameters

use clap::Args;
use config::{Config, File, FileFormat};
use serde::Deserialize;
use wnrake::{
    cache::Cache,
    client::Client,
    proxy::{Api, Proxy},
};

#[derive(Args, Clone, Debug, Deserialize)]
#[group(required = false, multiple = true)]
pub struct ConfigParams {
    /// Solver URL [default: http://localhost:8191]
    #[arg(long, value_name = "URL")]
    solver: Option<String>,

    /// Disable cache
    #[arg(long)]
    disable_cache: Option<bool>,

    /// Cache [default: disabled]
    #[arg(long, value_name = "DIR")]
    cache: Option<String>,

    /// Proxy
    #[arg(long, value_name = "URL")]
    proxy: Option<String>,

    /// Proxy username
    #[arg(long, value_name = "USERNAME")]
    proxy_username: Option<String>,

    /// Proxy password
    #[arg(long, value_name = "PASSWORD")]
    proxy_password: Option<String>,

    /// Proxy API
    #[arg(long, value_name = "URL")]
    proxy_api: Option<String>,

    /// Proxy API username
    #[arg(long, value_name = "USERNAME")]
    proxy_api_username: Option<String>,

    /// Proxy API password
    #[arg(long, value_name = "PASSWORD")]
    proxy_api_password: Option<String>,

    /// Proxy API key
    #[arg(long, value_name = "KEY")]
    proxy_api_key: Option<String>,
}

impl ConfigParams {
    /// Fills any None values with the configuration file defaults
    pub fn load_config(&mut self, config: &str) {
        log::info!("Using configuration {}", config);
        match Config::builder()
            .add_source(File::new(config, FileFormat::Toml))
            .build()
        {
            Ok(settings) => match settings.try_deserialize::<ConfigParams>() {
                Ok(default_params) => {
                    if self.solver.is_none() {
                        self.solver = default_params.solver;
                    }
                    if !self.disable_cache.unwrap_or(false) && self.cache.is_none() {
                        self.cache = if default_params.disable_cache.unwrap_or(false) {
                            None
                        } else {
                            default_params.cache
                        };
                    }
                    if self.proxy.is_none() {
                        self.proxy = default_params.proxy;
                    }
                    if self.proxy_username.is_none() {
                        self.proxy_username = default_params.proxy_username;
                    }
                    if self.proxy_password.is_none() {
                        self.proxy_password = default_params.proxy_password;
                    }
                    if self.proxy_api.is_none() {
                        self.proxy_api = default_params.proxy_api;
                    }
                    if self.proxy_api_username.is_none() {
                        self.proxy_api_username = default_params.proxy_api_username;
                    }
                    if self.proxy_api_password.is_none() {
                        self.proxy_api_password = default_params.proxy_api_password;
                    }
                    if self.proxy_api_key.is_none() {
                        self.proxy_api_key = default_params.proxy_api_key;
                    }
                }
                Err(e) => log::warn!("{:?}", e),
            },
            Err(e) => log::warn!("{:?}", e),
        }
    }

    fn configure_proxy(&self) -> Option<Proxy> {
        match &self.proxy {
            Some(url) => {
                let api_impl = match self.proxy_api.as_deref() {
                    Some(url) => {
                        if let Some(key) = self.proxy_api_key.as_deref() {
                            Some(Api::with_api_key(url, key))
                        } else if let (Some(username), Some(password)) = (
                            self.proxy_api_username.as_deref(),
                            self.proxy_api_password.as_deref(),
                        ) {
                            Some(Api::with_basic_auth(url, username, password))
                        } else {
                            Some(Api::new(url))
                        }
                    }
                    None => None,
                };
                let mut proxy = Proxy::builder(url);
                if let Some(username) = self.proxy_username.as_deref() {
                    proxy = proxy.username(username);
                    if let Some(password) = self.proxy_password.as_deref() {
                        proxy = proxy.password(password);
                    }
                }
                if let Some(api) = api_impl {
                    proxy = proxy.api(api);
                }
                Some(proxy.build())
            }
            None => None,
        }
    }

    fn configure_cache(&self) -> Option<Cache> {
        match &self.cache {
            Some(dir) => match Cache::new(dir) {
                Ok(cache) => Some(cache),
                Err(e) => {
                    log::warn!("{:?}", e);
                    None
                }
            },
            None => None,
        }
    }

    pub fn to_client(&self) -> Client {
        let solver = self.solver.as_deref().unwrap_or("http://localhost:8191");
        let mut client = Client::builder(solver);
        if let Some(proxy) = self.configure_proxy() {
            client = client.proxy(proxy);
        }
        if let Some(cache) = self.configure_cache() {
            client = client.cache(cache);
        }
        client.build()
    }
}
