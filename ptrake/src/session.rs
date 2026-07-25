//! debug command

use crate::error::Error;
use crawler::Client;
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
        Session { client }
    }

    pub async fn login(&mut self, email: &str, password: &str) -> Result<(), Error> {
        self.client.create_session().await?;
        let solution = self.client.get("https://patreon.com/login").await?;
        match solution.status {
            200 => {

/*
[
    {
        "domain" : "www.patreon.com",
        "expiry": 1800496537,
        "httpOnly" : false,
        "name" : "g_state",
        "path" : "/",
        "sameSite" : "Lax",
        "secure" : false,
        "value" : "{\"i_l\":0,\"i_ll\":1784944537026,\"i_b\":\"elkXo2K7769B7/WoxaLfOUTbVzddmGpw80f9wi6rWSk\",\"i_e\":{\"enable_itp_optimization\":24},\"i_et\":1784944537026}"
    }, 
    {
        "domain" : ".patreon.com",
        "expiry" : 1784946336,
        "httpOnly" : true,
        "name" : "__cf_bm",
        "path" : "/",
        "sameSite" : "None",
        "secure" : true,
        "value" : "bbH3LX1jmxvKtOH5n4Ap0VHusJAXXMoQfBGJtAl2P0A-1784944536.563358-1.0.1.1-WkYihF0JPiYSBON3XfRgt6Mn2WP0oahWN7G_KQtQ8m0cDCCGvjSYSCyo0WioT6eBIMmcQ.gkCSRTld7wu4WumBDfJ0Ht1OyHuokEk9x8lpXWgVKRYOg3hv_pBSy_ZQdjjhhXJLWpc0jZmt.jsMn6GA")
    },
    {
        "domain" : "www.patreon.com",
        "expiry" : 1819504536,
        "httpOnly" : false,
        "name" : "patreon_locale_code",
        "path" : "/",
        "sameSite" : "Lax",
        "secure" : true,
        "value" : "en-US"
    },
    {
        "domain" : "www.patreon.com",
        "expiry" : 1819504536,
        "httpOnly" : false,
        "name" : "patreon_location_country_code",
        "path" : "/",
        "sameSite" : "Lax",
        "secure" : true,
        "value" : "US"
    },
    {
        "domain" : "www.patreon.com",
        "expiry" : 1819504536,
        "httpOnly" : false,
        "name" : "patreon_device_id",
        "path" : "/",
        "sameSite" : "Lax",
        "secure" : true,
        "value" : "c77cba5d-7376-4cd1-aa8b-8c125915af1c"
    }
]
 */


                log::debug!("USER_AGENT: {}", solution.user_agent);
                log::debug!("HEADERS: {:?}", solution.headers);
                log::debug!("COOKIES: {:?}", solution.cookies);
                Ok(())
            }
            status => Err(Error::login("failed to load login page")),
        }

        /*
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
        */
    }

    pub async fn logout(&mut self) -> Result<(), Error> {
        Ok(self.client.destroy_session().await?)
    }
}
