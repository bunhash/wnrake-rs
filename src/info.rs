//! info command

use crate::config::ConfigParams;
use clap::Args;
use wnrake::{
    book::BookInfo,
    client::Client,
    error::Error,
    parser::{Downloader, Parser, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Info {
    url: Option<String>,
}

impl Info {
    async fn do_work(&self, client: &mut Client) -> Result<(), Error> {
        // Load bookinfo
        let mut title = None;
        let mut author = None;
        let url = match &self.url {
            Some(url) => {
                log::info!("ignoring bookinfo.txt");
                url.clone()
            }
            None => {
                log::info!("reading from bookinfo.txt");
                let bookinfo = BookInfo::from_file("bookinfo.txt")?;
                log::info!("Current book info:");
                log::info!("  {}", &bookinfo.title);
                log::info!("  {}", &bookinfo.author);
                log::info!("  {}", &bookinfo.url);
                title = Some(bookinfo.title);
                author = Some(bookinfo.author);
                bookinfo.url
            }
        };

        // Load parser
        let parser = WnParser::try_from(url.as_str())?;
        log::info!("using parser {:?}", parser);

        // Fetch bookinfo (overwrite with local values)
        let res = parser.get_book_info(client, &url).await?;
        let mut bookinfo = parser.parse_book_info(&url, &res)?;
        log::info!("Found book info:");
        log::info!("  {}", &bookinfo.title);
        log::info!("  {}", &bookinfo.author);
        log::info!("  {}", &bookinfo.url);
        if title.is_some() {
            bookinfo.title = title.expect("title should not be None");
            log::info!("using title: {}", &bookinfo.title);
        }
        if author.is_some() {
            bookinfo.author = author.expect("author should not be None");
            log::info!("using author: {}", &bookinfo.author);
        }

        // Write bookinfo
        log::info!("writing to bookinfo.txt");
        bookinfo.to_file("bookinfo.txt")?;

        // Fetch url cache
        log::info!("fetching chapter list");
        let url_cache = parser.get_chapterlist(client, &res).await?;
        log::info!("found {} chapters", url_cache.0.len());

        // Write url cache
        log::info!("writing to urlcache.txt");
        url_cache.to_file("urlcache.txt")?;

        // Print results
        println!("Title: {}", &bookinfo.title);
        println!("Author: {}", &bookinfo.author);
        println!("URL: {}", &bookinfo.url);
        println!("Chapters: {}", url_cache.0.len());

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
