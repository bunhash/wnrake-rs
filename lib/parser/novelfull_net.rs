//! novelfull.net parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::Client,
    error::Error,
    parser::{utils, Downloader, Parser},
    request::{Request, WaitFor},
};
use async_trait::async_trait;
use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink, Selector};

#[derive(Clone, Debug)]
pub struct NovelFullNetParser;

#[async_trait]
impl Downloader for NovelFullNetParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(Request::get(url).wait_for(WaitFor::class("info")).build())
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div.info")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid book info page", false)),
        }
    }

    async fn get_chapterlist(
        &self,
        client: &mut Client,
        _url: &str,
        html: &str,
    ) -> Result<UrlCache, Error> {
        let mut res = html.to_string();

        // Get all chapter URLs
        let mut chapterlist = UrlCache::new();
        loop {
            let url = {
                let document = Html::parse_document(&res);
                for a in document.select(&Selector::parse("div#list-chapter div.row li a")?) {
                    log::debug!("found: {:?}", a);
                    let uri = a
                        .attr("href")
                        .ok_or(Error::html("expected href attribute in a", true))?;
                    chapterlist.0.push(format!("https://novelfull.net{}", uri));
                }
                match document
                    .select(&Selector::parse("ul.pagination li.next a")?)
                    .next()
                {
                    Some(a) => {
                        let uri = a
                            .attr("href")
                            .ok_or(Error::html("expected href attribute in a", true))?;
                        Some(format!("https://novelfull.net{}", uri))
                    }
                    None => None,
                }
            };
            if url.is_none() {
                break;
            }
            res = client
                .request(
                    Request::get(&url.unwrap())
                        .wait_for(WaitFor::class("div#list-chapter"))
                        .build(),
                )
                .await?;
        }

        // Return chapters
        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("chapter-content"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("div#chapter-content")?)
            .next()
        {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for NovelFullNetParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        // Get title
        let title_h3 = document
            .select(&Selector::parse("h3.title")?)
            .next()
            .ok_or(Error::html("expected h3.title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");
        let author_div = document
            .select(&Selector::parse("div.info > div")?)
            .next()
            .ok_or(Error::html("expected author div", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        // Return book info
        Ok(BookInfo {
            title: title_h3.trim().into(),
            author: author_div
                .trim()
                .trim_start_matches("Author:")
                .trim()
                .into(),
            url: url.into(),
        })
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        let document = Html::parse_document(&html);

        // Kill some stuff first
        let script_ids = document
            .select(&Selector::parse("script")?)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        let sink = HtmlTreeSink::new(document);
        for id in script_ids {
            sink.remove_from_parent(&id);
        }
        let document = sink.finish();

        // Parse title
        let title = document
            .select(&Selector::parse("a.chapter-title")?)
            .next()
            .ok_or(Error::html("expected a.chapter-title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        // Get chapter content
        let chapter = document
            .select(&Selector::parse("div#chapter-content")?)
            .next()
            .ok_or(Error::html("chapter-content not in html", true))?;

        // Build HTML
        let html = utils::parse_content(&title, chapter)?;

        // Return chapter
        Ok(Chapter {
            title: title.trim().into(),
            html,
        })
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        let document = Html::parse_document(html);
        match document.select(&Selector::parse("a#next_chap")?).next() {
            Some(el) => match el.attr("href") {
                Some(uri) => Ok(Some(format!("https://novelfull.net{}", uri))),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}
