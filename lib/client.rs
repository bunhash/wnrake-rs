//! Flaresolverr Client

use crate::{
    cache::Cache,
    error::{Error, ErrorType},
    proxy::Proxy,
    solution::Response,
};
use serde_json::{Map, Number, Value};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    solver: String,
    timeout: Duration,
    proxy: Option<Proxy>,
    session: Option<String>,
    cache: Option<Cache>,
}

impl Client {
    /// Creates a new Client
    pub(crate) fn new_internal(
        solver: String,
        timeout: Duration,
        proxy: Option<Proxy>,
        cache: Option<Cache>,
    ) -> Client {
        let client = reqwest::Client::new();
        Client {
            client,
            solver,
            timeout,
            proxy,
            session: None,
            cache,
        }
    }

    pub fn builder(solver: &str) -> ClientBuilder {
        ClientBuilder::new(solver)
    }

    /// Creates a new Client
    pub fn new(solver: &str) -> Client {
        Client::new_internal(solver.into(), Duration::from_secs(60), None, None)
    }

    /// Creates a new Client with a proxy configuration
    pub fn with_proxy(solver: &str, proxy: Proxy) -> Client {
        Client::new_internal(solver.into(), Duration::from_secs(60), Some(proxy), None)
    }

    /// Get solver URL
    pub fn solver(&self) -> &str {
        self.solver.as_ref()
    }

    /// Get timeout
    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    /// Get proxy
    pub fn proxy(&self) -> Option<&Proxy> {
        self.proxy.as_ref()
    }

    /// Get session
    pub fn session(&self) -> Option<&str> {
        self.session.as_deref()
    }

    /// Get cache handler
    pub fn cache(&self) -> Option<&Cache> {
        self.cache.as_ref()
    }

    async fn post_data(&self, data: &Map<String, Value>) -> Result<Response, Error> {
        // Send HTTP Post
        let res = self.client.post(&self.solver).json(&data).send().await?;
        log::debug!("solver response: {:?}", &res);

        // Parse JSON
        let res = res.json::<Response>().await.map_err(Error::json)?;

        // Get the status
        if res.status == "ok" {
            Ok(res)
        } else {
            log::debug!("solution error {:?}", &res);
            Err(Error::solution(&res.message))
        }
    }

    /// Starts a flaresolverr session
    pub async fn create_session(&mut self) -> Result<(), Error> {
        // Build the JSON request
        // { "cmd" : "sessions.create", "proxy" : { "url" : <proxy> } }
        let mut data = Map::new();
        data.insert("cmd".into(), Value::String("sessions.create".into()));
        if let Some(proxy) = &self.proxy {
            data.insert("proxy".into(), proxy.to_json());
        }

        // Send command
        let res = self.post_data(&data).await?;
        match res.session {
            Some(session) => {
                log::debug!("created session: {}", &session);
                self.session = Some(session);
                Ok(())
            }
            None => Err(Error::solution("no session in response")),
        }
    }

    /// Ends the flaresolverr session
    pub async fn destroy_session(&mut self) -> Result<(), Error> {
        if let Some(session) = &self.session {
            // Build the JSON request
            // { "cmd" : "sessions.destory", "session" : <session> }
            let mut data = Map::new();
            data.insert("cmd".into(), Value::String("sessions.destroy".into()));
            data.insert("session".into(), Value::String(session.into()));

            // Send command
            self.client.post(&self.solver).json(&data).send().await?;
            log::debug!("destroyed session: {}", &session);
            self.session = None;
        }
        Ok(())
    }

    async fn get_single(&mut self, url: &str, xpath: Option<&str>) -> Result<String, Error> {
        // Build the JSON request
        // { "cmd" : "request.get", "url" : <url>, "maxTimeout" : <timeout>, "session" : <session> }
        let mut data = Map::new();
        data.insert("cmd".into(), Value::String("request.get".into()));
        data.insert("url".into(), Value::String(url.into()));
        data.insert(
            "maxTimeout".into(),
            Value::Number(Number::from_u128(self.timeout.as_millis()).expect("bad duration value")),
        );
        if let Some(val) = xpath {
            data.insert("xpath".into(), Value::String(val.into()));
        }
        if let Some(session) = &self.session {
            data.insert("session".into(), Value::String(session.into()));
        }

        // Send command
        let res = self.post_data(&data).await?;
        match res.solution {
            Some(solution) => {
                if solution.status == 200 {
                    Ok(solution.response)
                } else {
                    Err(Error::status(solution.status))
                }
            }
            None => Err(Error::solution("no solution in response")),
        }
    }

    async fn fetch(&mut self, url: &str, xpath: Option<&str>) -> Result<String, Error> {
        log::debug!("fetching {}", url);
        let max_attempts = 3;
        let mut cur_attempt = 1;
        loop {
            match self.get_single(url, xpath).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    // If try again
                    if e.error_type != ErrorType::Solution || cur_attempt >= max_attempts {
                        return Err(e);
                    }

                    log::warn!(
                        "Attempt ({}/{}): failed to download {}",
                        cur_attempt,
                        max_attempts,
                        url
                    );

                    // Reset session and proxy
                    log::debug!("Solution error. Resetting session");
                    self.destroy_session().await?;
                    if let Some(proxy) = &self.proxy {
                        log::debug!("Resetting proxy");
                        proxy.restart().await?;
                    }
                    self.create_session().await?;
                }
            }
            log::debug!("failed attempts: {}", cur_attempt);
            cur_attempt = cur_attempt + 1;
        }
    }

    /// Does a HTTP GET on the URL, solving a CloudFlare challenge, if necessary. Will attempt 3
    /// times.
    pub async fn get(&mut self, url: &str, xpath: Option<&str>) -> Result<String, Error> {
        let has_cache = self.cache.is_some();
        if has_cache {
            let res = self.cache.as_ref().expect("cache should exist").get(url)?;
            match res {
                Some(res) => {
                    log::debug!("{} found in cache", url);
                    Ok(res)
                }
                None => {
                    log::debug!("{} not found in cache", url);
                    let res = self.fetch(url, xpath).await?;
                    self.cache
                        .as_ref()
                        .expect("cache should exist")
                        .insert(url, res.as_bytes())?;
                    Ok(res)
                }
            }
        } else {
            self.fetch(url, xpath).await
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    solver: String,
    timeout: Duration,
    proxy: Option<Proxy>,
    cache: Option<Cache>,
}

impl ClientBuilder {
    pub fn new(solver: &str) -> Self {
        ClientBuilder {
            solver: solver.into(),
            timeout: Duration::from_secs(60),
            proxy: None,
            cache: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn build(self) -> Client {
        Client::new_internal(self.solver, self.timeout, self.proxy, self.cache)
    }
}
