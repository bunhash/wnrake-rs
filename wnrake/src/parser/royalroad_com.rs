//! royalroad.com parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::WnrakeClient,
    error::Error,
    parser::{utils, Downloader, Parser},
};
use async_trait::async_trait;
use crawler::{Request, WaitFor};
use scraper::{Html, Selector};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct RoyalRoadParser;

#[async_trait]
impl Downloader for RoyalRoadParser {
    async fn get_book_info(&self, client: &mut WnrakeClient, url: &str) -> Result<String, Error> {
        let res = client
            .request(&Request::get(url).wait_for(WaitFor::id("chapters")).build())
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("#chapters")?).next() {
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
        let mut chapterlist = UrlCache::new();

        /* -- old html
        for row in document.select(&Selector::parse("#chapters tbody tr")?) {
            match row.select(&Selector::parse("a")?).next() {
                Some(link) => {
                    let uri = link.attr("href").ok_or(Error::html(
                        &format!("expected href in link: {:?}", link),
                        true,
                    ))?;
                    chapterlist
                        .0
                        .push(format!("https://www.royalroad.com{}", uri));
                }
                None => continue,
            }
        }
        */

        for row in document.select(&Selector::parse("script")?) {
            let script = row.text().collect::<Vec<_>>().join("");
            if script.contains("window.chapters") {
                for line in script.split('\n') {
                    if line.contains("window.chapters") {
                        let json: Value = serde_json::from_str(
                            line.trim()
                                .trim_start_matches("window.chapters = ")
                                .trim_end_matches(";"),
                        )
                        .map_err(Error::json)?;
                        if let Value::Array(objects) = &json {
                            let mut ordered = vec![None; objects.len()];
                            for obj in objects {
                                let index = obj["order"]
                                    .as_u64()
                                    .ok_or(Error::json("invalid chapter index"))?
                                    as usize;
                                let unlocked = obj["isUnlocked"]
                                    .as_bool()
                                    .ok_or(Error::json("invalid unlocked boolean"))?;
                                if unlocked && index < objects.len() {
                                    ordered[index] = Some(
                                        obj["url"]
                                            .as_str()
                                            .ok_or(Error::json("invalid chapter uri"))?,
                                    );
                                }
                            }
                            for url in ordered {
                                match url {
                                    Some(val) => chapterlist
                                        .0
                                        .push(format!("https://www.royalroad.com{}", val)),
                                    None => break,
                                }
                            }
                        }
                        break;
                    }
                }
                break;
            }
        }

        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut WnrakeClient, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                &Request::get(url)
                    .wait_for(WaitFor::selector("div.chapter-content"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("div.chapter-content")?)
            .next()
        {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for RoyalRoadParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        let story_div = document
            .select(&Selector::parse("div.fic-title")?)
            .next()
            .ok_or(Error::html("expected div.fic-title", true))?;
        let title_h1 = story_div
            .select(&Selector::parse("h1")?)
            .next()
            .ok_or(Error::html("expected title h1", true))?;
        let spans = story_div
            .select(&Selector::parse("span")?)
            .collect::<Vec<_>>();
        let author_span = spans
            .get(1)
            .ok_or(Error::html("expected author span", true))?;

        let title = title_h1.text().collect::<Vec<_>>().join("");
        let author = author_span.text().collect::<Vec<_>>().join("");

        // Return book info
        Ok(BookInfo {
            title: title.trim().into(),
            author: author.trim().into(),
            url: url.into(),
        })
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        let document = Html::parse_document(&html);

        // Get title
        let title = document
            .select(&Selector::parse("div.fic-header h1")?)
            .next()
            .ok_or(Error::html("expected div.fic-header h1", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Get chapter div
        let chapter = document
            .select(&Selector::parse("div.chapter-content")?)
            .next()
            .ok_or(Error::html("expected div.chapter-content", true))?;

        // Build HTML
        let html = utils::parse_content(&title, chapter)?;

        /*
        // Build HTML
        let html = format!(
            "<html><head></head><body><h1>{}</h1>{}</body></html>",
            title,
            utils::parse_content(chapter)?,
        );
        */

        // Return chapter
        Ok(Chapter { title, html })
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        let document = Html::parse_document(html);
        Ok(
            match document.select(&Selector::parse("div.nav-buttons")?).next() {
                Some(nav_div) => {
                    let links = nav_div.select(&Selector::parse("a")?).collect::<Vec<_>>();
                    match links.last() {
                        Some(link) => {
                            let text = link.text().collect::<Vec<_>>().join("");
                            match text.contains("Next") {
                                true => {
                                    let uri = link
                                        .attr("href")
                                        .ok_or(Error::html("no href in link", true))?;
                                    Some(format!("https://www.royalroad.com{}", uri))
                                }
                                false => None,
                            }
                        }
                        None => None,
                    }
                }
                None => None,
            },
        )
    }
}
