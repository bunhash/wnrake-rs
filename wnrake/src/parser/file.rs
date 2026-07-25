//! ranobes.net parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::WnrakeClient,
    error::Error,
    parser::{Downloader, Parser},
};
use async_trait::async_trait;
use std::fs;

#[derive(Clone, Debug)]
pub struct FileParser;

#[async_trait]
impl Downloader for FileParser {
    async fn get_book_info(&self, _: &mut WnrakeClient, _: &str) -> Result<String, Error> {
        Err(Error::parser("not implemented for file"))
    }

    async fn get_chapterlist(
        &self,
        _: &mut WnrakeClient,
        _: &str,
        _: &str,
    ) -> Result<UrlCache, Error> {
        Err(Error::parser("not implemented for file"))
    }

    async fn get_chapter(&self, _: &mut WnrakeClient, url: &str) -> Result<String, Error> {
        let path = url
            .trim()
            .trim_start_matches("file:")
            .trim_start_matches("/");
        log::info!("Reading file: {}", path);
        Ok(fs::read_to_string(path)?)
    }
}

impl Parser for FileParser {
    fn parse_book_info(&self, _: &str, _: &str) -> Result<BookInfo, Error> {
        Err(Error::parser("not implemented for file"))
    }

    fn parse_chapter(&self, _html: &str) -> Result<Chapter, Error> {
        Err(Error::parser("not implemented for file"))
    }

    fn next_page(&self, _: &str) -> Result<Option<String>, Error> {
        Err(Error::parser("not implemented for file"))
    }
}
