//! wanderinginn.com parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::WnrakeClient,
    error::Error,
    parser::{utils, Downloader, Parser},
};
use async_trait::async_trait;
use crawler::{Request, WaitFor};
use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct WanderingInnParser;

#[async_trait]
impl Downloader for WanderingInnParser {
    async fn get_book_info(&self, client: &mut WnrakeClient, _url: &str) -> Result<String, Error> {
        let res = client
            .request(
                &Request::get("https://wanderinginn.com/table-of-contents/")
                    .wait_for(WaitFor::id("table-of-contents"))
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("#table-of-contents")?)
            .next()
        {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid book info page", false)),
        }
    }

    async fn get_chapterlist(
        &self,
        _client: &mut WnrakeClient,
        _url: &str,
        html: &str,
    ) -> Result<UrlCache, Error> {
        let document = Html::parse_document(&html);
        let toc = document
            .select(&Selector::parse("#table-of-contents")?)
            .next()
            .ok_or(Error::html("expected #table-of-contents", true))?;
        let mut chapterlist = UrlCache::new();
        for chapter in toc.select(&Selector::parse("div.chapter-entry")?) {
            let url = chapter
                .select(&Selector::parse("a")?)
                .next()
                .ok_or(Error::html("expected a", true))?
                .attr("href")
                .ok_or(Error::html("no href in link", true))?;
            chapterlist.0.push(url.into());
        }

        // Return chapters
        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut WnrakeClient, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                &Request::get(url)
                    .wait_for(WaitFor::id("reader-content"))
                    .build(),
            )
            .await?;

        let document = Html::parse_document(&res);
        let reader_content = document
            .select(&Selector::parse("#reader-content")?)
            .next()
            .ok_or(Error::html("reader-content not in html", true))?;
        let _ = reader_content
            .select(&Selector::parse("article")?)
            .next()
            .ok_or(Error::html("patreon access blocked", true))?;

        // All good
        Ok(res)
    }
}

impl Parser for WanderingInnParser {
    fn parse_book_info(&self, _url: &str, _html: &str) -> Result<BookInfo, Error> {
        Ok(BookInfo {
            title: "The Wandering Inn".into(),
            author: "Pirateaba".into(),
            url: "https://wanderinginn.com/table-of-contents/".into(),
        })
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        let document = Html::parse_document(&html);

        // Parse title
        let title = document
            .select(&Selector::parse("meta[property=\"og:title\"]")?)
            .next()
            .ok_or(Error::html("expected meta[property-\"og:title\"]", true))?
            .attr("content")
            .ok_or(Error::html("no content in meta tag", true))?;

        // Get chapter content
        let reader_content = document
            .select(&Selector::parse("#reader-content")?)
            .next()
            .ok_or(Error::html("reader-content not in html", true))?;

        let chapter = reader_content
            .select(&Selector::parse("article")?)
            .next()
            .ok_or(Error::html("article not in html", true))?;

        // Build HTML
        let html = utils::parse_content(&title, chapter)?;

        // Return chapter
        Ok(Chapter {
            title: title.trim().into(),
            html,
        })
    }

    fn next_page(&self, _html: &str) -> Result<Option<String>, Error> {
        Err(Error::parser("Not implemented--do wnrake info"))
    }
}
