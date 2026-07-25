//! debug command

use crate::{client::WnrakeClient, error::Error};
use clap::Args;
use crawler::config::Config;
use std::io::{self, Write};

#[derive(Args, Clone, Debug)]
pub struct Debug;

impl Debug {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        let mut client = WnrakeClient::from_config(config)?;

        log::debug!("Solver={}", client.client.solver());
        log::debug!("Proxy={:?}", client.client.proxy());
        log::debug!("Cache={:?}", &client.cache);

        client.client.create_session().await?;
        let mut buffer = String::new();
        io::stdout().write(b"Press [Enter] ")?;
        io::stdout().flush()?;
        let input = io::stdin();
        input.read_line(&mut buffer)?;
        client.client.destroy_session().await?;
        Ok(())
    }
}
