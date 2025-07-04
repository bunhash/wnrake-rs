//! Flaresolverr Client

use crate::{
    error::{Error, ErrorType},
    proxy::Proxy,
    solution::Response,
};
use serde_json::{Map, Number, Value};
use std::{fmt, time::Duration};

#[derive(Clone, Debug)]
pub struct Client<'a> {
    client: reqwest::Client,
    solver: String,
    timeout: Duration,
    proxy: Option<&'a Proxy>,
    session: Option<String>,
}

impl fmt::Display for Client<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Client<'_> {
    /// Creates a new Client
    pub(crate) fn new_internal<'a>(
        solver: String,
        timeout: Duration,
        proxy: Option<&'a Proxy>,
    ) -> Client<'a> {
        let client = reqwest::Client::new();
        Client {
            client,
            solver,
            timeout,
            proxy,
            session: None,
        }
    }

    /// Creates a new Client
    pub fn new(solver: &str) -> Client<'_> {
        Client::new_internal(solver.into(), Duration::new(60, 0), None)
    }

    /// Creates a new Client with a proxy configuration
    pub fn with_proxy<'a>(solver: &str, proxy: &'a Proxy) -> Client<'a> {
        Client::new_internal(solver.into(), Duration::new(60, 0), Some(proxy))
    }

    async fn post_data(&self, data: &Map<String, Value>) -> Result<Response, Error> {
        // Send HTTP Post
        let res = self.client.post(&self.solver).json(&data).send().await?;

        // Parse JSON
        let res = res.json::<Response>().await.map_err(Error::json)?;

        // Get the status
        if res.status == "ok" {
            Ok(res)
        } else {
            Err(Error::solution(&res.message))
        }
    }

    /// Starts a flaresolverr session
    pub async fn create_session(&mut self) -> Result<(), Error> {
        // Build the JSON request
        // { "cmd" : "sessions.create", "proxy" : { "url" : <proxy> } }
        let mut data = Map::new();
        data.insert("cmd".into(), Value::String("sessions.create".into()));
        if let Some(proxy) = self.proxy {
            data.insert("proxy".into(), proxy.to_json());
        }

        // Send command
        let res = self.post_data(&data).await?;
        match res.session {
            Some(session) => {
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
            self.session = None;
        }
        Ok(())
    }

    async fn get_single(&mut self, url: &str) -> Result<String, Error> {
        // Build the JSON request
        // { "cmd" : "request.get", "url" : <url>, "maxTimeout" : <timeout>, "session" : <session> }
        let mut data = Map::new();
        data.insert("cmd".into(), Value::String("request.get".into()));
        data.insert("url".into(), Value::String(url.into()));
        data.insert(
            "maxTimeout".into(),
            Value::Number(Number::from_u128(self.timeout.as_millis()).expect("bad duration value")),
        );
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

    /// Does a HTTP GET on the URL, solving a CloudFlare challenge, if necessary. Will attempt 3
    /// times.
    pub async fn get(&mut self, url: &str) -> Result<String, Error> {
        let max_attempts = 3;
        let mut cur_attempt = 1;
        loop {
            match self.get_single(url).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    // If try again
                    if e.error_type != ErrorType::Solution || cur_attempt >= max_attempts {
                        return Err(e);
                    }
                    cur_attempt = cur_attempt + 1;

                    // Reset proxy
                    self.destroy_session().await?;
                    if let Some(proxy) = self.proxy {
                        proxy.restart().await?;
                    }
                    self.create_session().await?;
                }
            }
        }
    }
}
