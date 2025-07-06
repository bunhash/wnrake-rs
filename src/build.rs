//! build command

use crate::config::Config;
use clap::Args;
use wnrake::{error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Build {
    url: Option<String>,
}

impl Build {
    pub async fn execute<'a>(&self, _config: &Config) -> Result<(), Error> {
        log::debug!("Build {:?}", self.url);
        Ok(())
    }
}
