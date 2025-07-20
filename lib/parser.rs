//! Parser implementations

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::Client,
    error::Error,
};
use async_trait::async_trait;
use reqwest::Url;

mod fanfiction_net;
mod phrases;
mod ranobes_top;
mod royalroad_com;
mod scribblehub_com;
mod utils;

pub use fanfiction_net::FanfictionParser;
pub use ranobes_top::RanobesParser;
pub use royalroad_com::RoyalRoadParser;
pub use scribblehub_com::ScribbleHubParser;

/// Trait for webnovel parsers
#[async_trait]
pub trait Downloader {
    /// Returns the novel's landing page HTML
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error>;

    /// Returns a list of URLs for each chapter (in order)
    async fn get_chapterlist(
        &self,
        client: &mut Client,
        url: &str,
        html: &str,
    ) -> Result<UrlCache, Error>;

    /// Returns the chapter's HTML
    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error>;
}

pub trait Parser {
    /// Parses the HTML of the novel's landing page
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error>;

    /// Parses the HTML of the novel's chapter
    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error>;

    /// Parses the HTML of the current page and returns the URL of the next chapter's page
    fn next_page(&self, html: &str) -> Result<Option<String>, Error>;
}

#[derive(Clone, Debug)]
pub enum WnParser {
    Fanfiction(FanfictionParser),
    Ranobes(RanobesParser),
    RoyalRoad(RoyalRoadParser),
    ScribbleHub(ScribbleHubParser),
}

impl TryFrom<&str> for WnParser {
    type Error = Error;
    fn try_from(url: &str) -> Result<WnParser, Self::Error> {
        let url = Url::parse(url).map_err(Error::solver)?;
        match url.domain() {
            Some("fanfiction.net") => Ok(WnParser::Fanfiction(FanfictionParser)),
            Some("www.fanfiction.net") => Ok(WnParser::Fanfiction(FanfictionParser)),
            Some("ranobes.top") => Ok(WnParser::Ranobes(RanobesParser)),
            Some("www.ranobes.top") => Ok(WnParser::Ranobes(RanobesParser)),
            Some("royalroad.com") => Ok(WnParser::RoyalRoad(RoyalRoadParser)),
            Some("www.royalroad.com") => Ok(WnParser::RoyalRoad(RoyalRoadParser)),
            Some("scribblehub.com") => Ok(WnParser::ScribbleHub(ScribbleHubParser)),
            Some("www.scribblehub.com") => Ok(WnParser::ScribbleHub(ScribbleHubParser)),
            _ => Err(Error::parser(format!("invalid url: {}", url))),
        }
    }
}

#[async_trait]
impl Downloader for WnParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.get_book_info(client, url).await,
            WnParser::Ranobes(parser) => parser.get_book_info(client, url).await,
            WnParser::RoyalRoad(parser) => parser.get_book_info(client, url).await,
            WnParser::ScribbleHub(parser) => parser.get_book_info(client, url).await,
        }
    }

    async fn get_chapterlist(
        &self,
        client: &mut Client,
        url: &str,
        html: &str,
    ) -> Result<UrlCache, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.get_chapterlist(client, url, html).await,
            WnParser::Ranobes(parser) => parser.get_chapterlist(client, url, html).await,
            WnParser::RoyalRoad(parser) => parser.get_chapterlist(client, url, html).await,
            WnParser::ScribbleHub(parser) => parser.get_chapterlist(client, url, html).await,
        }
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.get_chapter(client, url).await,
            WnParser::Ranobes(parser) => parser.get_chapter(client, url).await,
            WnParser::RoyalRoad(parser) => parser.get_chapter(client, url).await,
            WnParser::ScribbleHub(parser) => parser.get_chapter(client, url).await,
        }
    }
}

impl Parser for WnParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.parse_book_info(url, html),
            WnParser::Ranobes(parser) => parser.parse_book_info(url, html),
            WnParser::RoyalRoad(parser) => parser.parse_book_info(url, html),
            WnParser::ScribbleHub(parser) => parser.parse_book_info(url, html),
        }
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.parse_chapter(html),
            WnParser::Ranobes(parser) => parser.parse_chapter(html),
            WnParser::RoyalRoad(parser) => parser.parse_chapter(html),
            WnParser::ScribbleHub(parser) => parser.parse_chapter(html),
        }
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        match self {
            WnParser::Fanfiction(parser) => parser.next_page(html),
            WnParser::Ranobes(parser) => parser.next_page(html),
            WnParser::RoyalRoad(parser) => parser.next_page(html),
            WnParser::ScribbleHub(parser) => parser.next_page(html),
        }
    }
}
