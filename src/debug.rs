//! debug command

use clap::Args;
use std::io::{self, Write};
use wnrake::{config::Config, error::Error};

#[derive(Args, Clone, Debug)]
pub struct Debug;

impl Debug {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        let mut client = config.to_client()?;

        log::debug!("Solver={}", client.solver());
        log::debug!("Proxy={:?}", client.proxy());
        log::debug!("Cache={:?}", client.cache());

        client.create_session().await?;
        let mut buffer = String::new();
        io::stdout().write(b"Press [Enter] ")?;
        io::stdout().flush()?;
        let input = io::stdin();
        input.read_line(&mut buffer)?;
        client.destroy_session().await?;
        Ok(())
    }
}
