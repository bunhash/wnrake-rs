//! Flaresolverr Client

use crate::{cache::Cache, error::Error, proxy::Proxy, solution::Response};
use serde_json::{Map, Number, Value};
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum WaitFor {
    Id(String),
    XPath(String),
    Link(String),
    ExactLink(String),
    Name(String),
    Tag(String),
    Class(String),
    Selector(String),
}

impl WaitFor {
    pub fn id(val: &str) -> Self {
        WaitFor::Id(val.into())
    }

    pub fn xpath(val: &str) -> Self {
        WaitFor::XPath(val.into())
    }

    pub fn link(val: &str) -> Self {
        WaitFor::Link(val.into())
    }

    pub fn exact_link(val: &str) -> Self {
        WaitFor::ExactLink(val.into())
    }

    pub fn name(val: &str) -> Self {
        WaitFor::Name(val.into())
    }

    pub fn tag(val: &str) -> Self {
        WaitFor::Tag(val.into())
    }

    pub fn class(val: &str) -> Self {
        WaitFor::Class(val.into())
    }

    pub fn selector(val: &str) -> Self {
        WaitFor::Selector(val.into())
    }

    /// Returns a reference to the type name
    pub fn type_name(&self) -> &str {
        match self {
            WaitFor::Id(_) => "id",
            WaitFor::XPath(_) => "xpath",
            WaitFor::Link(_) => "link",
            WaitFor::ExactLink(_) => "exact-link",
            WaitFor::Name(_) => "name",
            WaitFor::Tag(_) => "tag",
            WaitFor::Class(_) => "class",
            WaitFor::Selector(_) => "selector",
        }
    }

    /// Returns a reference to the value
    pub fn value(&self) -> &str {
        match self {
            WaitFor::Id(v) => v.as_str(),
            WaitFor::XPath(v) => v.as_str(),
            WaitFor::Link(v) => v.as_str(),
            WaitFor::ExactLink(v) => v.as_str(),
            WaitFor::Name(v) => v.as_str(),
            WaitFor::Tag(v) => v.as_str(),
            WaitFor::Class(v) => v.as_str(),
            WaitFor::Selector(v) => v.as_str(),
        }
    }
}

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
            Err(Error::parse_solution_error(&res.message))
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

    pub async fn get(&mut self, url: &str, wait_for: Option<&WaitFor>) -> Result<String, Error> {
        self._get(url, wait_for, true).await
    }

    pub async fn get_or_kill(
        &mut self,
        url: &str,
        wait_for: Option<&WaitFor>,
    ) -> Result<String, Error> {
        self._get(url, wait_for, false).await
    }

    /// Does a HTTP GET on the URL, solving a CloudFlare challenge, if necessary. Will attempt 3
    /// times.
    async fn _get(
        &mut self,
        url: &str,
        wait_for: Option<&WaitFor>,
        no_kill: bool,
    ) -> Result<String, Error> {
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
                    let res = self.n_attempts(url, wait_for, no_kill, 5).await?;
                    self.cache
                        .as_ref()
                        .expect("cache should exist")
                        .insert(url, res.as_bytes())?;
                    Ok(res)
                }
            }
        } else {
            self.fetch(url, wait_for, no_kill).await
        }
    }

    /// Recovers by resetting the session and restarting the proxy (if possible)
    pub async fn recover(&mut self, seconds: u64) -> Result<(), Error> {
        self.destroy_session().await?;
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.restart(seconds).await?;
        }
        self.create_session().await
    }

    async fn n_attempts(
        &mut self,
        url: &str,
        wait_for: Option<&WaitFor>,
        no_kill: bool,
        max_attempts: usize,
    ) -> Result<String, Error> {
        log::debug!("fetching {}", url);
        let mut attempts = 0;
        loop {
            // Send command
            match self.fetch(url, wait_for, no_kill).await {
                Ok(res) => return Ok(res),
                Err(e) => match e.fatal {
                    true => {
                        log::error!("fatal: {}", e);
                        return Err(e);
                    }
                    false => {
                        attempts = attempts + 1;
                        log::error!("({}/{}) attempts: {}", attempts, max_attempts, e);
                        if attempts >= max_attempts {
                            return Err(e);
                        }
                    }
                },
            }

            // Recover
            loop {
                match self.recover(60).await {
                    Ok(_) => break,
                    Err(e) => match e.fatal {
                        true => {
                            log::error!("fatal: {}", e);
                            return Err(e);
                        }
                        false => {
                            attempts = attempts + 1;
                            log::error!("({}/{}) attempts: {}", attempts, max_attempts, e);
                            if attempts >= max_attempts {
                                return Err(e);
                            }
                        }
                    },
                }
            }
        }
    }

    async fn fetch(
        &mut self,
        url: &str,
        wait_for: Option<&WaitFor>,
        no_kill: bool,
    ) -> Result<String, Error> {
        // Build the JSON request
        // { "cmd" : "request.get", "url" : <url>, "maxTimeout" : <timeout>, "session" : <session> }
        let mut data = Map::new();
        data.insert("cmd".into(), Value::String("request.get".into()));
        data.insert("url".into(), Value::String(url.into()));
        data.insert(
            "maxTimeout".into(),
            Value::Number(Number::from_u128(self.timeout.as_millis()).expect("bad duration value")),
        );
        if let Some(wf) = wait_for {
            data.insert("waitType".into(), wf.type_name().into());
            data.insert("waitFor".into(), wf.value().into());
        }
        if let Some(session) = &self.session {
            data.insert("session".into(), Value::String(session.into()));
        }
        if no_kill {
            data.insert("noKill".into(), "true".into());
        }
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
