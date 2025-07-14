//! Flaresolverr Client

use crate::{
    cache::Cache,
    error::Error,
    proxy::Proxy,
    request::{Request, Session},
    response::Response,
};
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

    /// Starts a flaresolverr session
    pub async fn create_session(&mut self) -> Result<(), Error> {
        let json = Session::create(self.proxy.as_ref());
        let res = self.client.post(&self.solver).json(&json).send().await?;
        log::debug!("solver response: {:?}", &res);

        // Parse JSON
        let res = res.json::<Response>().await.map_err(Error::json)?;
        if res.status == "ok" {
            match res.session {
                Some(session) => {
                    log::debug!("created session: {}", &session);
                    self.session = Some(session);
                    Ok(())
                }
                None => Err(Error::solution("no session in response")),
            }
        } else {
            log::debug!("solution error {:?}", &res);
            Err(Error::parse_solution_error(&res.message))
        }
    }

    /// Ends the flaresolverr session
    pub async fn destroy_session(&mut self) -> Result<(), Error> {
        if let Some(session) = &self.session {
            let json = Session::destroy(session);
            let _ = self.client.post(&self.solver).json(&json).send().await;
            log::debug!("destroyed session: {}", &session);
            self.session = None;
        }
        Ok(())
    }

    /// Processes the flaresolverr request
    pub async fn request(&mut self, mut request: Request) -> Result<String, Error> {
        request.max_timeout = self.timeout.as_millis();
        request.session = self.session.clone();
        let attempts = request.attempts;
        if request.enable_cache && self.cache.is_some() {
            let res = self
                .cache
                .as_ref()
                .expect("cache should exist")
                .get(&request.url)?;
            match res {
                Some(res) => {
                    log::debug!("{} found in cache", request.url);
                    Ok(res)
                }
                None => {
                    log::debug!("{} not found in cache", request.url);
                    let res = self.n_requests(&mut request, attempts).await?;
                    self.cache
                        .as_ref()
                        .expect("cache should exist")
                        .insert(&request.url, res.as_bytes())?;
                    Ok(res)
                }
            }
        } else {
            Ok(self.n_requests(&mut request, attempts).await?)
        }
    }

    /// Convenience function for the typical HTTP GET
    pub async fn get(&mut self, url: &str) -> Result<String, Error> {
        self.request(Request::get(url).build()).await
    }

    /// Convenience function for the typical HTTP POST
    pub async fn post(&mut self, url: &str, post_data: &[(&str, &str)]) -> Result<String, Error> {
        self.request(Request::post(url).post_data(post_data).build())
            .await
    }

    /// Attempt to recover by resetting the session (and reconnecting the VPN)
    pub async fn recover(&mut self, seconds: u64) -> Result<(), Error> {
        self.destroy_session().await?;
        if let Some(proxy) = &self.proxy {
            proxy.restart(seconds).await?;
        }
        self.create_session().await
    }

    /// Attempts the request and will recover from non-fatal errors up to N times
    async fn n_requests(
        &mut self,
        request: &Request,
        max_attempts: usize,
    ) -> Result<String, Error> {
        let mut attempts = 0;
        loop {
            // Send command
            match self._request(request).await {
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

    /// Gets the response, expects a good solution and a HTTP 200 response
    async fn _request(&self, request: &Request) -> Result<String, Error> {
        // Send HTTP Post
        let res = self.client.post(&self.solver).json(request).send().await?;
        log::debug!("solver response: {:?}", &res);

        // Parse JSON
        let res = res.json::<Response>().await.map_err(Error::json)?;

        // Get the status
        if res.status == "ok" {
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
        } else {
            log::debug!("solution error {:?}", &res);
            Err(Error::parse_solution_error(&res.message))
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
