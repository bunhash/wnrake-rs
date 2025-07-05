//! build command

use crate::config::ConfigParams;
use clap::Args;
use wnrake::{error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Build {
    url: Option<String>,
}

impl Build {
    pub async fn execute<'a>(&self, _params: &ConfigParams) -> Result<(), Error> {
        log::info!("Build {:?}", self.url);
        Ok(())
    }
}
