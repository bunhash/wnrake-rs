//! ranobes.top parser

use crate::{
    book::{BookInfo, Chapter, UrlCache},
    client::Client,
    error::Error,
    parser::{Downloader, Parser},
    request::{Request, WaitFor},
};
use async_trait::async_trait;
use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink, Selector};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct RanobesParser;

#[async_trait]
impl Downloader for RanobesParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("dle-content"))
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("div.r-fullstory-s1")?)
            .next()
        {
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
        let (total_toc_pages, more_chapters) = {
            let document = Html::parse_document(&html);

            // Calculate the total TOC pages (25 chapters per page)
            let total_chapters = self.get_total_chapters(&document)?;
            log::debug!("Total chapters: {}", total_chapters);
            let total_toc_pages = total_chapters
                .checked_add(24)
                .ok_or(Error::html("bad chapter count", true))?
                .checked_div(25)
                .ok_or(Error::html("bad chapter count", true))?;
            log::debug!("Total TOC pages: {}", total_toc_pages);

            // Get TOC base URL
            let more_chapters = document
                .select(&Selector::parse("div.r-fullstory-chapters-foot a")?)
                .nth(1)
                .ok_or(Error::html("no footer links found", true))?
                .attr("href")
                .ok_or(Error::html("no href in link", true))?;
            (
                total_toc_pages,
                format!("https://ranobes.top{}", more_chapters),
            )
        };

        // Get all chapter URLs
        let mut chapterlist = UrlCache::new();
        let mut url = more_chapters.clone();
        for page in 0..total_toc_pages {
            let res = client.get(&url).await?;
            let doc = Html::parse_document(&res);
            for script in doc.select(&Selector::parse("script")?) {
                let text = script.text().collect::<Vec<_>>().join("");
                if text.contains("window.__DATA__") {
                    let json: Value =
                        serde_json::from_str(text.trim().trim_start_matches("window.__DATA__ = "))
                            .map_err(Error::json)?;
                    match &json["chapters"] {
                        Value::Array(chapters) => {
                            for chapter in chapters {
                                chapterlist.0.insert(
                                    0,
                                    chapter["link"]
                                        .as_str()
                                        .ok_or(Error::json("no link in chapter"))?
                                        .into(),
                                )
                            }
                        }
                        _ => return Err(Error::json("no chapters in json")),
                    }
                    break;
                }
            }
            url = format!("{}page/{}/", &more_chapters, page + 2);
        }

        // Return chapters
        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("arrticle"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div#arrticle")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for RanobesParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        // Get h1.title
        let title_h1 = document
            .select(&Selector::parse("div.r-fullstory-s1 h1.title")?)
            .next()
            .ok_or(Error::html("expected title.h1", true))?;
        let span_selector = Selector::parse("span")?;
        let mut spans = title_h1.select(&span_selector);
        let span_0 = spans
            .next()
            .ok_or(Error::html("expected fullstory span[0]", true))?;
        let span_1 = spans
            .next()
            .ok_or(Error::html("expected fullstory span[1]", true))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        // Get author
        let author = span_1.trim().trim_start_matches("by").trim();

        // Get title
        let title = if span_0.attr("hidden").is_some() {
            title_h1
                // Immediate children
                .children()
                // Filter only Text
                .filter_map(|c| c.value().as_text())
                // Grab first
                .next()
                .ok_or(Error::html("expected text", true))?
                // Convert to string and trim
                .to_string()
                .trim()
                .to_string()
        } else {
            span_0
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string()
        };

        // Return book info
        Ok(BookInfo {
            title,
            author: author.into(),
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
        let ad_ids = document
            .select(&Selector::parse("div.free-support-top")?)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        let sink = HtmlTreeSink::new(document);
        for id in script_ids {
            sink.remove_from_parent(&id);
        }
        for id in ad_ids {
            sink.remove_from_parent(&id);
        }
        let document = sink.finish();

        // Parse title
        let title: String = match document
            .select(&Selector::parse(r#"h1[class="h4 title"]"#)?)
            .next()
        {
            Some(c) => match c
                .children()
                .filter_map(|c| match c.value().as_text() {
                    Some(t) => {
                        let text = t.trim();
                        if text.is_empty() {
                            None
                        } else {
                            Some(text.to_string())
                        }
                    }
                    None => None,
                })
                .next()
            {
                Some(text) => text.into(),
                None => "???".into(),
            },
            None => "???".into(),
        };

        // Parse the chapter
        let chapter = document
            .select(&Selector::parse("div#arrticle")?)
            .next()
            .ok_or(Error::html("arrticle not in html", true))?;

        // Build HTML
        let html = format!(
            "<html><body><h1>{}</h1>{}</body></html>",
            title,
            chapter.inner_html().trim()
        );

        // Return chapter
        Ok(Chapter { title, html })
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        let document = Html::parse_document(html);
        match document.select(&Selector::parse("a#next")?).next() {
            Some(el) => Ok(Some(
                el.attr("href")
                    .ok_or(Error::html("no href in link", false))?
                    .into(),
            )),
            None => Ok(None),
        }
    }
}

impl RanobesParser {
    fn get_total_chapters(&self, html: &Html) -> Result<u32, Error> {
        // Get first ul in r-fullstory-spec
        let novel_spec_ul = html
            .select(&Selector::parse("div.r-fullstory-spec ul")?)
            .next()
            .ok_or(Error::html("no r-fullstory-spec uls", true))?;

        // Cycle through lis to find the span
        let mut num_chapters_span = None;
        for li in novel_spec_ul.select(&Selector::parse("li")?) {
            let text = li.text().collect::<Vec<_>>().join("");
            if text.contains("Available") || text.contains("Translated") {
                let selector = Selector::parse("span")?;
                num_chapters_span = Some(
                    li.select(&selector)
                        .next()
                        .ok_or(Error::html("no span in novel_spec_items", true))?,
                );
            }
        }

        // Get the chapter count and convert to u32
        let num_chapters = num_chapters_span
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join("");
        let num_chapters = num_chapters.trim_end_matches("chapters").trim();
        u32::from_str_radix(num_chapters, 10).map_err(|e| Error::html(e, true))
    }
}
