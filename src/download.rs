//! download command

use crate::config::ConfigParams;
use clap::Args;
use wnrake::{error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Download {
    url: Option<String>,
}

impl Download {
    pub async fn execute<'a>(&self, params: &ConfigParams) -> Result<(), Error> {
        let client = params.to_client();
        log::info!("Solver={}", client.solver());
        log::info!("Proxy={:?}", client.proxy());
        log::info!("Cache={:?}", client.cache());
        log::info!("Download {:?}", self.url);
        Ok(())
    }
}
