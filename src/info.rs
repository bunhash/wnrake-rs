//! info command

use clap::Args;
use wnrake::{
    book::BookInfo,
    client::Client,
    config::Config,
    error::Error,
    parser::{Downloader, Parser, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Info {
    url: Option<String>,
}

impl Info {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        let mut client = config.to_client()?;

        log::debug!("Solver={}", client.solver());
        log::debug!("Proxy={:?}", client.proxy());
        log::debug!("Cache={:?}", client.cache());

        client.create_session().await?;
        let res = self.do_work(&mut client).await;
        client.destroy_session().await?;
        res
    }

    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        // Load bookinfo
        let mut title = None;
        let mut author = None;
        let url = match &self.url {
            Some(url) => {
                log::debug!("ignoring bookinfo.txt");
                url.clone()
            }
            None => {
                log::debug!("reading from bookinfo.txt");
                let bookinfo = BookInfo::from_file("bookinfo.txt")
                    .or(Err(Error::io("bookinfo.txt does not exit")))?;
                log::debug!("Current book info:");
                log::debug!("  {}", &bookinfo.title);
                log::debug!("  {}", &bookinfo.author);
                log::debug!("  {}", &bookinfo.url);
                title = Some(bookinfo.title);
                author = Some(bookinfo.author);
                bookinfo.url
            }
        };

        // Load parser
        let parser = WnParser::try_from(url.as_str())?;
        log::debug!("using parser {:?}", parser);

        // Fetch bookinfo (overwrite with local values)
        let res = parser.get_book_info(client, &url).await?;
        let mut bookinfo = parser.parse_book_info(&url, &res)?;
        log::debug!("Found book info:");
        log::debug!("  {}", &bookinfo.title);
        log::debug!("  {}", &bookinfo.author);
        log::debug!("  {}", &bookinfo.url);
        if title.is_some() {
            bookinfo.title = title.expect("title should not be None");
            log::debug!("using title: {}", &bookinfo.title);
        }
        if author.is_some() {
            bookinfo.author = author.expect("author should not be None");
            log::debug!("using author: {}", &bookinfo.author);
        }

        // Write bookinfo
        log::debug!("writing to bookinfo.txt");
        bookinfo.to_file("bookinfo.txt")?;

        // Fetch url cache
        log::debug!("fetching chapter list");
        let url_cache = parser.get_chapterlist(client, &res).await?;
        log::debug!("found {} chapters", url_cache.0.len());

        // Write url cache
        log::debug!("writing to urlcache.txt");
        url_cache.to_file("urlcache.txt")?;

        // Print results
        log::info!("Title: {}", &bookinfo.title);
        log::info!("Author: {}", &bookinfo.author);
        log::info!("URL: {}", &bookinfo.url);
        log::info!("Chapters: {}", url_cache.0.len());

        Ok(())
    }
}
