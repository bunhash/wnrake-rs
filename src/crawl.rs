//! crawl command

use crate::config::ConfigParams;
use clap::Args;
use wnrake::{client::Client, error::Error, parser::WnParser};

#[derive(Args, Clone, Debug)]
pub struct Crawl {
    url: Option<String>,
}

impl Crawl {
    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        Ok(())
    }

    pub async fn execute<'a>(&self, params: &ConfigParams) -> Result<(), Error> {
        let mut client = params.to_client();

        log::info!("Solver={}", client.solver());
        log::info!("Proxy={:?}", client.proxy());
        log::info!("Cache={:?}", client.cache());

        client.create_session().await?;
        let res = self.do_work(&mut client).await;
        client.destroy_session().await?;
        res
    }
}
