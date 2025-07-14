//! royalroad.com parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::Client,
    error::Error,
    parser::{Downloader, Parser},
    request::{Request, WaitFor},
};
use async_trait::async_trait;
use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct RoyalRoadParser;

#[async_trait]
impl Downloader for RoyalRoadParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(Request::get(url).wait_for(WaitFor::id("chapters")).build())
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("#chapters")?).next() {
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
        let mut chapterlist = UrlCache::new();
        for row in document.select(&Selector::parse("#chapters tr")?) {
            match row.select(&Selector::parse("a")?).next() {
                Some(link) => {
                    let uri = link
                        .attr("href")
                        .ok_or(Error::html("expected href in link", true))?;
                    chapterlist
                        .0
                        .push(format!("https://www.royalroad.com{}", uri));
                }
                None => continue,
            }
        }
        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
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

        let title = document
            .select(&Selector::parse("div.fic-header h1")?)
            .next()
            .ok_or(Error::html("expected div.fic-header h1", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let chapter = document
            .select(&Selector::parse("div.chapter-content")?)
            .next()
            .ok_or(Error::html("expected div.chapter-content", true))?;

        let mut paragraphs = Vec::new();
        for child in chapter.select(&Selector::parse("p,div")?) {
            let text = child.text().collect::<Vec<_>>().join("").trim().to_string();
            if !text.is_empty() {
                paragraphs.push(child.html());
            }
        }
        if paragraphs.is_empty() {
            let new_chapter = chapter
                .html()
                .replace("<br>", "\n")
                .replace("<br/>", "\n")
                .replace("</br>", "");
            let chapter = Html::parse_fragment(&new_chapter);
            let text = chapter
                .select(&Selector::parse("div")?)
                .next()
                .ok_or(Error::html("div should exist", true))?
                .text()
                .collect::<Vec<_>>()
                .join("");
            for line in text.lines() {
                let para = line.trim();
                if !para.is_empty() {
                    paragraphs.push(format!("<p>{}</p>", para));
                }
            }
        }

        // Build HTML
        let html = format!(
            "<html><body><h1>{}</h1>{}</body></html>",
            title,
            paragraphs.join("")
        );

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
