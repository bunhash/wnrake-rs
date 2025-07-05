//! parse command

use crate::config::ConfigParams;
use clap::Args;
use wnrake::{error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Parse {
    url: Option<String>,
}

impl Parse {
    pub async fn execute<'a>(&self, _params: &ConfigParams) -> Result<(), Error> {
        log::info!("Parse {:?}", self.url);
        Ok(())
    }
}
