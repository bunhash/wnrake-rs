//! debug command

use crate::error::Error;
use crawler::client::Client;
use scraper::{Html, Selector};
use serde_json::json;

const AUTH_URL: &str = "https://patreon.com/api/auth?include=user.null&fields[user]=[]&json-api-version=1.0&json-api-use-default-includes=false";

pub struct Session {
    client: Client,
}

impl Session {
    /// Creates a new session with email/password logon
    pub fn new(client: Client) -> Self {
        log::debug!("Solver={}", client.solver());
        log::debug!("Proxy={:?}", client.proxy());
        log::debug!("Cache={:?}", client.cache());
        Session { client }
    }

    pub async fn login(&mut self, email: &str, password: &str) -> Result<(), Error> {
        self.client.create_session().await?;
        let _ = self.client.get("https://patreon.com/login").await?;
        let _ = self
            .client
            .json(
                AUTH_URL,
                json!({
                    "data" : {
                        "type" : "genericPatreonApi",
                        "attributes" : {
                            "patreon_auth" : {
                                "email" : email,
                                "password" : password,
                                "allow_account_creation" : false
                            },
                            "auth_context" : "auth",
                            "ru" : "https://www.patreon.com/home"
                        },
                        "relationships" : {}
                    }
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<(), Error> {
        Ok(self.client.destroy_session().await?)
    }
}
