//! download command

use crate::{
    config::{Config, ProxyConfig},
    utils,
};
use clap::Args;
use std::{fs::File, io::Write, path::Path};
use wnrake::{
    book::UrlCache,
    client::Client,
    error::Error,
    parser::{Downloader, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Download {
    /// Use multiple threads [one for each configured proxy]
    #[arg(short = 't', long)]
    use_threads: bool,
}

impl Download {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        if self.use_threads {
            let proxies = config
                .load_config_file()
                .proxies
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<ProxyConfig>>();
            for proxy in proxies {
                log::debug!("{:?}", proxy);
            }
            Ok(())
        } else {
            let mut client = config.to_client();

            log::debug!("Solver={}", client.solver());
            log::debug!("Proxy={:?}", client.proxy());
            log::debug!("Cache={:?}", client.cache());

            client.create_session().await?;
            let res = self.do_work(&mut client).await;
            client.destroy_session().await?;
            res
        }
    }

    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        // Make staging directory
        utils::ensure_staging()?;

        // Load URL cache
        log::debug!("loading urlcache.txt");
        let url_cache = UrlCache::from_file("urlcache.txt")?;
        let total_chapters = url_cache.as_ref().len();
        log::debug!("total chapters: {}", total_chapters);

        for (i, url) in url_cache.as_ref().iter().enumerate() {
            // Get path
            let filename = utils::to_filename(i, url);
            let path = Path::join(Path::new("staging"), &filename);

            match path.is_file() {
                true => {
                    log::info!("({:>4}/{:>4}) Using cached {}", i + 1, total_chapters, url);
                }
                false => {
                    // Load parser
                    let parser = WnParser::try_from(url.as_str())?;
                    log::debug!("using parser {:?}", parser);

                    // Download
                    log::info!("({:>4}/{:>4}) Downloading {}", i + 1, total_chapters, url);
                    let chapter = parser.get_chapter(client, url).await?;

                    // Write file
                    let mut file = File::create(path)?;
                    file.write_all(chapter.as_bytes())?;
                }
            }
        }
        Ok(())
    }
}
