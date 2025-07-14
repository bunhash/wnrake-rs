//! scribblehub.com parser

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
pub struct ScribbleHubParser;

#[async_trait]
impl Downloader for ScribbleHubParser {
    async fn get_book_info(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::class("fic_title"))
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document.select(&Selector::parse("div.fic_title")?).next() {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid book info page", false)),
        }
    }

    async fn get_chapterlist(
        &self,
        client: &mut Client,
        url: &str,
        _: &str,
    ) -> Result<UrlCache, Error> {
        // Need a fresh page for the POST ID
        //
        // Cookies:
        // - Set TOC to 50 chapters per page
        // - Set order to Ascending
        //
        let html = client
            .request(
                Request::get(url)
                    .cookies(&[("toc_show", "50"), ("toc_sorder", "asc")])
                    .disable_cache()
                    .build(),
            )
            .await?;

        // POST ID -- needed to ask for chapters
        let (mypostid, total_chapters) = {
            let document = Html::parse_document(&html);
            let mypostid = document
                .select(&Selector::parse("#mypostid")?)
                .next()
                .ok_or(Error::html("expected id=mypostid", true))?
                .attr("value")
                .ok_or(Error::html("expected value of mypostid", true))?
                .to_string();
            log::debug!("mypostid={}", mypostid);
            let total_chapters = self.get_total_chapters(&document)?;
            log::debug!("Total chapters: {}", total_chapters);
            (mypostid, total_chapters)
        };

        // Get total chapter count to calculate the number of TOC pages
        let toc_pages = total_chapters
            .checked_add(49)
            .ok_or(Error::html("bad chapter count", true))?
            .checked_div(50)
            .ok_or(Error::html("bad chapter count", true))?;
        log::debug!("TOC pages: {}", toc_pages);

        let mut chapterlist = UrlCache::new();

        for i in 0..toc_pages {
            let res = client
                .post(
                    "https://www.scribblehub.com/wp-admin/admin-ajax.php",
                    &[
                        ("action", "wi_getreleases_pagination"),
                        ("pagenum", &format!("{}", i + 1)),
                        ("mypostid", &mypostid),
                    ],
                )
                .await?;
            let fragment = Html::parse_fragment(&res);
            for link in fragment.select(&Selector::parse("a")?) {
                chapterlist.0.push(
                    link.attr("href")
                        .ok_or(Error::html("expected href in link", true))?
                        .into(),
                );
            }
        }

        Ok(chapterlist)
    }

    async fn get_chapter(&self, client: &mut Client, url: &str) -> Result<String, Error> {
        let res = client
            .request(
                Request::get(url)
                    .wait_for(WaitFor::id("main read chapter"))
                    .with_kill()
                    .build(),
            )
            .await?;
        let document = Html::parse_document(&res);
        match document
            .select(&Selector::parse("main[id='main read chapter']")?)
            .next()
        {
            Some(_) => Ok(res),
            None => Err(Error::html("invalid chapter page", false)),
        }
    }
}

impl Parser for ScribbleHubParser {
    fn parse_book_info(&self, url: &str, html: &str) -> Result<BookInfo, Error> {
        let document = Html::parse_document(&html);

        // Get title
        let title = document
            .select(&Selector::parse("div.fic_title")?)
            .next()
            .ok_or(Error::html("expected div.fic_title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Get author
        let author = document
            .select(&Selector::parse("span.auth_name_fic")?)
            .next()
            .ok_or(Error::html("expected span.auth_name_fic", true))?
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

        // Get title
        let title = document
            .select(&Selector::parse("div.chapter-title")?)
            .next()
            .ok_or(Error::html("expected div.chapter-title", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Get chapter
        let paragraphs = document
            .select(&Selector::parse("div#chp_raw")?)
            .next()
            .ok_or(Error::html("expected id=chp_raw", true))?
            .child_elements()
            .map(|child| child.html())
            .collect::<Vec<_>>();

        // Build HTML
        let html = format!(
            "<html><body><h1>{}</h1>{}</body></html>",
            title,
            paragraphs.join(""),
        );

        // Return chapter
        Ok(Chapter { title, html })
    }

    fn next_page(&self, html: &str) -> Result<Option<String>, Error> {
        let document = Html::parse_document(html);
        Ok(
            match document.select(&Selector::parse("a.btn-next")?).next() {
                Some(link) => match link.attr("class") {
                    Some(class) => match class.contains("disabled") {
                        true => None,
                        false => Some(
                            link.attr("href")
                                .ok_or(Error::html("expected href in link", true))?
                                .into(),
                        ),
                    },
                    None => Some(
                        link.attr("href")
                            .ok_or(Error::html("expected href in link", true))?
                            .into(),
                    ),
                },
                None => None,
            },
        )
    }
}

impl ScribbleHubParser {
    fn get_total_chapters(&self, html: &Html) -> Result<u32, Error> {
        let num_chapters = html
            .select(&Selector::parse("div.toc span.cnt_toc")?)
            .next()
            .ok_or(Error::html("expected span.cnt_toc", true))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        u32::from_str_radix(&num_chapters, 10).map_err(|e| Error::html(e, true))
    }
}
