//! crawl command

use crate::{config::Config, utils};
use clap::Args;
use std::{fs::File, io::Write, path::Path};
use wnrake::{
    book::UrlCache,
    client::Client,
    error::Error,
    parser::{Downloader, Parser, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Crawl;

impl Crawl {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        let mut client = config.to_client();

        log::debug!("Solver={}", client.solver());
        log::debug!("Proxy={:?}", client.proxy());
        log::debug!("Cache={:?}", client.cache());

        client.create_session().await?;
        let res = self.do_work(&mut client).await;
        client.destroy_session().await?;
        res
    }

    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        // Make staging directory
        utils::ensure_staging()?;

        // Load URL cache
        log::debug!("loading urlcache.txt");
        let mut url_cache = UrlCache::from_file("urlcache.txt")?;

        let res = self.try_do_work(client, &mut url_cache).await;
        url_cache.to_file("urlcache.txt")?;
        res
    }

    async fn try_do_work(
        &self,
        client: &mut Client,
        url_cache: &mut UrlCache,
    ) -> Result<(), Error> {
        let url_cache = url_cache.as_mut();
        let total_chapters = url_cache.len();
        log::debug!("total chapters: {}", total_chapters);

        for i in 0..total_chapters - 1 {
            let url = &url_cache[i];

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

        let mut index = total_chapters - 1;
        let mut next_url = &url_cache[index];
        loop {
            // Get path
            let filename = utils::to_filename(index, next_url);
            let path = Path::join(Path::new("staging"), &filename);

            // Load parser
            let parser = WnParser::try_from(next_url.as_str())?;
            log::debug!("using parser {:?}", parser);

            // Download
            log::info!("({:>4}/????) Downloading {}", index + 1, next_url);
            let chapter = parser.get_chapter(client, &next_url).await?;

            // Write file
            let mut file = File::create(path)?;
            file.write_all(chapter.as_bytes())?;

            // Get next page
            index = index + 1;
            match parser.next_page(&chapter)? {
                Some(url) => {
                    url_cache.push(url);
                    next_url = url_cache.last().expect("UrlCache should not be empty");
                }
                None => break,
            }
        }

        Ok(())
    }
}
