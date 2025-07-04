//! Parser implementations

use crate::{
    book::{BookInfo, Chapter},
    client::Client,
    error::Error,
};
use async_trait::async_trait;
use reqwest::Url;

mod ranobes_top;

pub use ranobes_top::RanobesParser;

/// Trait for webnovel parsers
#[async_trait]
pub trait Downloader {
    /// Returns the novel's landing page HTML
    async fn get_book_info<'a>(&self, client: &mut Client<'a>, url: &str) -> Result<String, Error>;

    /// Returns a list of URLs for each chapter (in order)
    async fn get_chapterlist<'a>(
        &self,
        client: &mut Client<'a>,
        html: &str,
    ) -> Result<Vec<String>, Error>;

    /// Returns the chapter's HTML
    async fn get_chapter<'a>(&self, client: &mut Client<'a>, url: &str) -> Result<String, Error>;
}

pub trait Parser {
    /// Parses the HTML of the novel's landing page
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error>;

    /// Parses the HTML of the novel's chapter
    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error>;

    /// Parses the HTML of the current page and returns the URL of the next chapter's page
    fn next_page(&self, html: &str) -> Result<Option<String>, Error>;
}

pub enum WnParser {
    Ranobes(RanobesParser),
}

impl TryFrom<&str> for WnParser {
    type Error = Error;
    fn try_from(url: &str) -> Result<WnParser, Self::Error> {
        let url = Url::parse(url).map_err(Error::solver)?;
        match url.domain() {
            Some("ranobes.top") => Ok(WnParser::Ranobes(RanobesParser)),
            Some("www.ranobes.top") => Ok(WnParser::Ranobes(RanobesParser)),
            _ => Err(Error::parser(format!("invalid url: {}", url))),
        }
    }
}

#[async_trait]
impl Downloader for WnParser {
    async fn get_book_info<'a>(&self, client: &mut Client<'a>, url: &str) -> Result<String, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.get_book_info(client, url).await,
        }
    }

    async fn get_chapterlist<'a>(
        &self,
        client: &mut Client<'a>,
        html: &str,
    ) -> Result<Vec<String>, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.get_chapterlist(client, html).await,
        }
    }

    async fn get_chapter<'a>(&self, client: &mut Client<'a>, url: &str) -> Result<String, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.get_chapter(client, url).await,
        }
    }
}

impl Parser for WnParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.parse_book_info(url, html),
        }
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.parse_chapter(html),
        }
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        match self {
            WnParser::Ranobes(parser) => parser.next_page(html),
        }
    }
}
