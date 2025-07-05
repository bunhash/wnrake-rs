//! download command

use crate::config::ConfigParams;
use clap::Args;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use wnrake::{
    book::UrlCache,
    client::Client,
    error::Error,
    parser::{Downloader, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Download;

impl Download {
    fn ensure_staging() -> Result<(), Error> {
        let dir = Path::new("staging");
        if !dir.is_dir() {
            log::info!("creating staging directory");
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    fn to_filename(index: usize, url: &str) -> String {
        let filename = url.rsplit("/").next().unwrap_or("chapter").replace(" ", "");
        format!("{:04}-{}", index, filename)
    }

    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        // Make staging directory
        Download::ensure_staging()?;

        // Load URL cache
        log::info!("loading urlcache.txt");
        let url_cache = UrlCache::from_file("urlcache.txt")?;
        let total_chapters = url_cache.0.len();
        log::info!("total chapters: {}", total_chapters);

        for (i, url) in url_cache.0.iter().enumerate() {
            // Get path
            let filename = Download::to_filename(i, url);
            let path = Path::join(Path::new("staging"), &filename);

            match path.is_file() {
                true => {
                    println!("({:>4}/{:>4}) Using cached {}", i + 1, total_chapters, url);
                }
                false => {
                    // Load parser
                    let parser = WnParser::try_from(url.as_str())?;
                    log::info!("using parser {:?}", parser);

                    // Download
                    println!("({:>4}/{:>4}) Downloading {}", i + 1, total_chapters, url);
                    let chapter = parser.get_chapter(client, url).await?;

                    // Write file
                    let mut file = File::create(path)?;
                    file.write_all(chapter.as_bytes())?;
                }
            }
        }
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
