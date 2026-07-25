//! Flaresolverr Client

use crate::{
    error::Error,
    proxy::Proxy,
    request::{Request, RequestInternal, Session},
    response::{Response, Solution},
};

#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    solver: String,
    proxy: Option<Proxy>,
    session: Option<String>,
}

impl Client {
    /// Creates a new Client
    pub(crate) fn new_internal(solver: String, proxy: Option<Proxy>) -> Client {
        let client = reqwest::Client::new();
        Client {
            client,
            solver,
            proxy,
            session: None,
        }
    }

    pub fn builder(solver: &str) -> ClientBuilder {
        ClientBuilder::new(solver)
    }

    /// Creates a new Client
    pub fn new(solver: &str) -> Client {
        Client::new_internal(solver.into(), None)
    }

    /// Creates a new Client with a proxy configuration
    pub fn with_proxy(solver: &str, proxy: Proxy) -> Client {
        Client::new_internal(solver.into(), Some(proxy))
    }

    /// Get solver URL
    pub fn solver(&self) -> &str {
        self.solver.as_ref()
    }

    /// Get proxy
    pub fn proxy(&self) -> Option<&Proxy> {
        self.proxy.as_ref()
    }

    /// Get session
    pub fn session(&self) -> Option<&str> {
        self.session.as_deref()
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
    pub async fn request(&mut self, request: &Request) -> Result<Solution, Error> {
        // Send HTTP Post
        let req = RequestInternal {
            request: &request,
            session: self.session.as_deref(),
        };
        let res = self.client.post(&self.solver).json(&req).send().await?;
        log::debug!("solver response: {:?}", &res);

        // Parse JSON
        let res = res.json::<Response>().await.map_err(Error::json)?;

        // Get the status
        if res.status == "ok" {
            res.solution
                .ok_or(Error::solution("no solution in response"))
        } else {
            log::debug!("solution error {:?}", &res);
            Err(Error::parse_solution_error(&res.message))
        }
    }

    /// Convenience function for the typical HTTP GET
    pub async fn get(&mut self, url: &str) -> Result<Solution, Error> {
        self.request(&Request::get(url).build()).await
    }

    /// Convenience function for the typical HTTP POST
    pub async fn post(&mut self, url: &str, post_data: &[(&str, &str)]) -> Result<Solution, Error> {
        self.request(&Request::post(url).post_data(post_data).build())
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
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    solver: String,
    proxy: Option<Proxy>,
}

impl ClientBuilder {
    pub fn new(solver: &str) -> Self {
        ClientBuilder {
            solver: solver.into(),
            proxy: None,
        }
    }

    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn build(self) -> Client {
        Client::new_internal(self.solver, self.proxy)
    }
}
