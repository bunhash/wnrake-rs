//! debug command

use crate::{error::Error, session::Session};
use clap::Args;
use crawler::config::Config;
use scraper::{Html, Selector};
use std::io::{self, Write};

#[derive(Args, Clone, Debug)]
pub struct Debug;

impl Debug {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        let mut session = Session::new(config.to_client()?);
        session
            .login("novels@bhmail.me", "S@woXM9PkFdB^n77XWeP")
            .await?;
        let mut buffer = String::new();
        io::stdout().write(b"Press [Enter] ")?;
        io::stdout().flush()?;
        let input = io::stdin();
        input.read_line(&mut buffer)?;
        session.logout().await?;
        Ok(())
    }
}
