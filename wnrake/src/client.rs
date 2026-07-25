//! client

use crate::error::Error;
use crawler::{cache::Cache, config::Config, Client, Request};

#[derive(Clone, Debug)]
pub struct WnrakeClient {
    /// Client
    pub client: Client,

    /// Cache
    pub cache: Option<Cache>,

    /// Download attempts
    pub attempts: usize,
}

impl WnrakeClient {
    /// Build `WnrakeClient` from `Config`
    pub fn from_config(config: &Config) -> Result<Self, Error> {
        let cache = match config.has_cache() {
            true => Some(config.to_cache()?),
            false => None,
        };
        Ok(Self {
            client: config.to_client()?,
            cache,
            attempts: 5,
        })
    }

    /// Build `WnrakeClient` from `Config`
    pub fn from_config_with_proxy(config: &Config, proxy: &str) -> Result<Self, Error> {
        let cache = match config.has_cache() {
            true => Some(config.to_cache()?),
            false => None,
        };
        Ok(Self {
            client: config.to_client_with_proxy(proxy)?,
            cache,
            attempts: 5,
        })
    }

    /// Processes download request
    pub async fn request(&mut self, request: &Request) -> Result<String, Error> {
        let url = request.url.clone();
        let resource = match &self.cache {
            Some(cache) => match cache.get(&url)? {
                Some(res) => {
                    log::debug!("{} found in cache", &url);
                    Some(res)
                }
                None => {
                    log::debug!("{} not found in cache", &url);
                    None
                }
            },
            None => None,
        };
        let resource = resource.unwrap_or(self.n_requests(request).await?);
        if let Some(cache) = &mut self.cache {
            cache.insert(&url, resource.as_bytes())?;
        }
        Ok(resource)
    }

    #[inline]
    async fn n_requests(&mut self, request: &Request) -> Result<String, Error> {
        let mut attempts = 0;
        loop {
            match self._request(request).await {
                Ok(res) => return Ok(res),
                Err(mut e) => match e.fatal {
                    true => {
                        log::error!("fatal: {}", e);
                        return Err(e);
                    }
                    false => {
                        attempts = attempts + 1;
                        log::error!("({}/{}) attempts: {}", attempts, self.attempts, e);
                        if attempts >= self.attempts {
                            e.fatal = true;
                            return Err(e);
                        }
                        self.client.recover(60).await?;
                    }
                },
            }
        }
    }

    #[inline]
    async fn _request(&mut self, request: &Request) -> Result<String, Error> {
        match self.client.request(request).await {
            Ok(solution) => match solution.status {
                200 => Ok(solution.response),
                status => Err(Error::status(format!("returned HTTP status {}", status))),
            },
            Err(e) => Err(e.into()),
        }
    }

    /// Convenience function for the typical HTTP GET
    pub async fn get(&mut self, url: &str) -> Result<String, Error> {
        self.request(&Request::get(url).build()).await
    }

    /// Convenience function for the typical HTTP POST
    pub async fn post(&mut self, url: &str, post_data: &[(&str, &str)]) -> Result<String, Error> {
        self.request(&Request::post(url).post_data(post_data).build())
            .await
    }
}
