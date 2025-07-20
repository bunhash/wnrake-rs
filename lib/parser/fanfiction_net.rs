//! fanfiction.net parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::Client,
    error::Error,
    parser::{utils, Downloader, Parser},
    request::{Request, WaitFor},
};
use async_trait::async_trait;
use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct FanfictionParser;

#[async_trait]
impl Downloader for FanfictionParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("profile_top"))
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div#profile_top")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid book info page", false)),
        }
    }

    async fn get_chapterlist(
        &self,
        _client: &mut Client,
        _url: &str,
        html: &str,
    ) -> Result<UrlCache, Error> {
        let document = Html::parse_document(&html);
        let chap_select = document
            .select(&Selector::parse("select#chap_select")?)
            .next()
            .ok_or(Error::html("expected chap_select", true))?;
        let onchange = chap_select
            .attr("onchange")
            .ok_or(Error::html("expected onchange attr", true))?;

        // Get the URL building html
        let parts = onchange.split("+").collect::<Vec<_>>();
        if parts.len() < 3 {
            return Err(Error::html("expected more parts in onchange value", true));
        }
        let prefix = parts[0]
            .trim()
            .trim_start_matches("self.location = '")
            .trim_end_matches("'");
        let suffix = parts[2]
            .trim()
            .trim_start_matches("'")
            .trim_end_matches("';");

        // Build all URLs
        Ok(UrlCache(
            chap_select
                .select(&Selector::parse("option")?)
                .filter_map(|o| {
                    Some(format!(
                        "https://fanfiction.net{}{}{}",
                        prefix,
                        o.attr("value")?,
                        suffix
                    ))
                })
                .collect::<Vec<_>>(),
        ))
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("storytext"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div#storytext")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for FanfictionParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        let profile_top = document
            .select(&Selector::parse("div#profile_top")?)
            .next()
            .ok_or(Error::html("expected profile_top", true))?;

        // Get title
        let title = profile_top
            .select(&Selector::parse("b")?)
            .next()
            .ok_or(Error::html("expected b", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Get author
        let author = profile_top
            .select(&Selector::parse("a")?)
            .next()
            .ok_or(Error::html("expected a", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Return book info
        Ok(BookInfo {
            title,
            author,
            url: url.into(),
        })
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        let document = Html::parse_document(&html);

        // Parse title
        let title = document
            .select(&Selector::parse(
                r#"select#chap_select > option[selected=""]"#,
            )?)
            .next()
            .ok_or(Error::html("page should have selected chapter", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let chapter = document
            .select(&Selector::parse("div#storytext")?)
            .next()
            .ok_or(Error::html("storytext not in html", true))?;

        // Build HTML
        let html = utils::parse_content(&title, chapter)?;

        // Return chapter
        Ok(Chapter { title, html })
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        let document = Html::parse_document(&html);
        let chap_select_btns = document
            .select(&Selector::parse(
                "div#content_wrapper_inner > span > button",
            )?)
            .collect::<Vec<_>>();
        let next = if chap_select_btns.len() == 0 {
            return Err(Error::html("expected at least 1 button", true));
        } else if chap_select_btns.len() == 1 {
            let text = chap_select_btns[0].text().collect::<Vec<_>>().join("");
            if text.contains("Next") {
                chap_select_btns[0]
            } else {
                return Ok(None);
            }
        } else {
            chap_select_btns[1]
        }
        .attr("onclick")
        .ok_or(Error::html("expect onclick attr", true))?;
        Ok(Some(format!(
            "https://www.fanfiction.net{}",
            next.trim_start_matches("self.location='")
                .trim_end_matches("'")
        )))
    }
}
