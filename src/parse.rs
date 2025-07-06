//! parse command

use crate::config::Config;
use clap::Args;
use wnrake::{error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Parse {
    url: Option<String>,
}

impl Parse {
    pub async fn execute<'a>(&self, _config: &Config) -> Result<(), Error> {
        log::debug!("Parse {:?}", self.url);
        Ok(())
    }
}
