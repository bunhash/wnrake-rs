//! lightnovelpub.org parser

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
pub struct LightNovelPubParser;

#[async_trait]
impl Downloader for LightNovelPubParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::class("novel-info"))
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div.novel-info")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid book info page", false)),
        }
    }

    async fn get_chapterlist(
        &self,
        client: &mut Client,
        url: &str,
        _html: &str,
    ) -> Result<UrlCache, Error> {
        let toc_url = format!("{}/chapters/", url.trim_end_matches("/"));
        let page_count = {
            let res = client
                .request(
                    Request::get(&toc_url)
                        .wait_for(WaitFor::class("page-selector"))
                        .build(),
                )
                .await?;
            let document = Html::parse_document(&res);
            let page_selector_span = document
                .select(&Selector::parse("div.page-selector > span")?)
                .nth(1)
                .ok_or(Error::html("expected div.page-selector span[1]", true))?
                .text()
                .collect::<Vec<_>>()
                .join("");
            let page_count = u32::from_str_radix(
                page_selector_span.trim().trim_start_matches("of").trim(),
                10,
            )
            .map_err(|e| Error::html(e, true))?;
            page_count
        };
        log::debug!("Total TOC pages: {}", page_count);

        // Get all chapter URLs
        let mut chapterlist = UrlCache::new();
        for page in 0..page_count {
            let toc_page_url = format!("{}?page={}", &toc_url, page + 1);
            let res = client
                .request(
                    Request::get(&toc_page_url)
                        .wait_for(WaitFor::class("page-selector"))
                        .build(),
                )
                .await?;
            let toc_page = Html::parse_document(&res);
            for div in toc_page.select(&Selector::parse("div.chapters-grid > div")?) {
                log::debug!("found: {:?}", div);
                let onclick = div
                    .attr("onclick")
                    .ok_or(Error::html("expected onclick attribute in div", true))?;
                let uri = onclick
                    .trim_start_matches("location.href='")
                    .trim_end_matches("'");
                chapterlist
                    .0
                    .push(format!("https://lightnovelpub.org{}", uri));
            }
        }

        // Return chapters
        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::class("chapter-container"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("div.chapter-container")?)
            .next()
        {
            Some(container) => match container
                .select(&Selector::parse("div.protection-barrier")?)
                .next()
            {
                Some(_) => Err(Error::html("must log in to read", false)),
                None => Ok(res),
            },
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for LightNovelPubParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        // Get novel-info
        let novel_info = document
            .select(&Selector::parse("div.novel-info")?)
            .next()
            .ok_or(Error::html("expected div.novel-info", true))?;

        // Get title
        let title = novel_info
            .select(&Selector::parse("h1.novel-title")?)
            .next()
            .ok_or(Error::html("expected h1.novel-title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");
        let author = novel_info
            .select(&Selector::parse("p.novel-author")?)
            .next()
            .ok_or(Error::html("expected h1.novel-title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        // Return book info
        Ok(BookInfo {
            title: title.trim().into(),
            author: author.trim().trim_start_matches("Author:").trim().into(),
            url: url.into(),
        })
    }

    fn parse_chapter(&self, html: &str) -> Result<Chapter, Error> {
        let document = Html::parse_document(&html);

        // Parse title
        let title = document
            .select(&Selector::parse("h1.chapter-title")?)
            .next()
            .ok_or(Error::html("expected h1.chapter-title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        // Get chapter content
        let chapter = document
            .select(&Selector::parse("div#chapterText")?)
            .next()
            .ok_or(Error::html("chapterText not in html", true))?;

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
        match document.select(&Selector::parse("a.next-btn")?).next() {
            Some(el) => {
                let uri = el
                    .attr("href")
                    .ok_or(Error::html("no href in link", false))?;
                Ok(Some(format!("https://lightnovelpub.org{}", uri)))
            }
            None => Ok(None),
        }
    }
}
