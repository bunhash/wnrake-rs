//! parse command

use crate::utils;
use clap::Args;
use std::{
    fs::{File, read_to_string},
    io::Write,
    path::Path,
    sync::Arc,
};
use tokio::sync::Mutex;
use wnrake::{
    book::{ChapterInfo, ChapterList, UrlCache},
    config::Config,
    error::Error,
    parser::{Parser, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Parse;

impl Parse {
    pub async fn execute<'a>(&self, _config: &Config) -> Result<(), Error> {
        // Make book directory
        utils::ensure_dir("book")?;

        // Load URL cache
        log::debug!("loading urlcache.txt");
        let url_cache = UrlCache::from_file("urlcache.txt")?;
        let total_chapters = url_cache.as_ref().len();
        log::debug!("total chapters: {}", total_chapters);

        // Build results
        let titles: Arc<Mutex<Vec<Option<String>>>> =
            Arc::new(Mutex::new(vec![None; url_cache.as_ref().len()]));

        // Build workers
        let workers = url_cache
            .0
            .into_iter()
            .enumerate()
            .map(|(i, url)| Worker {
                total_chapters: total_chapters,
                index: i,
                url: url,
                titles: titles.clone(),
            })
            .collect::<Vec<_>>();

        // Do work
        let futures = workers
            .into_iter()
            .map(|worker| tokio::spawn(worker.do_work()))
            .collect::<Vec<_>>();

        // Wait for work to complete
        for future in futures.into_iter() {
            let _ = future.await;
        }

        // Build chapter list
        let chapters = titles.as_ref().lock().await;
        let chapter_list = ChapterList(
            chapters
                .iter()
                .enumerate()
                .map(|(index, title)| {
                    let parsed_filename = utils::index_to_filename(index);
                    let filename = Path::join(Path::new("book"), &parsed_filename);
                    ChapterInfo {
                        path: filename.as_os_str().to_str().expect("bad filename?").into(),
                        title: title.as_deref().unwrap_or("???").into(),
                    }
                })
                .collect::<Vec<_>>(),
        );

        // Write chapter list
        chapter_list.to_file("chapterlist.txt")
    }
}

#[derive(Clone, Debug)]
struct Worker {
    total_chapters: usize,
    index: usize,
    url: String,
    titles: Arc<Mutex<Vec<Option<String>>>>,
}

impl Worker {
    pub async fn do_work(self) {
        // Get downloaded chapter
        let raw_filename = utils::url_to_filename(self.index, &self.url);
        let raw_path = Path::join(Path::new("staging"), &raw_filename);

        match raw_path.is_file() {
            // File exists
            true => match || -> Result<String, Error> {
                // Load parser
                let parser = WnParser::try_from(self.url.as_str())?;
                log::debug!("using parser {:?}", parser);

                // Get parsed filename
                let parsed_filename = utils::index_to_filename(self.index);
                let parsed_path = Path::join(Path::new("book"), &parsed_filename);

                // Parse chapter
                log::info!(
                    "({:>4}/{:>4}) parsing {}",
                    self.index + 1,
                    self.total_chapters,
                    self.url
                );
                let html = read_to_string(raw_path)?;
                let chapter = parser.parse_chapter(&html)?;

                // Write chapter
                let mut file = File::create(parsed_path)?;
                file.write_all(chapter.html.as_bytes())?;

                // Return title
                Ok(chapter.title)
            }() {
                Ok(title) => {
                    // Populate results
                    let mut titles = self.titles.as_ref().lock().await;
                    titles[self.index] = Some(title);
                }
                Err(e) => log::error!(
                    "({:>4},{:>4}) failed to parse chapter: {:?}",
                    self.index + 1,
                    self.total_chapters,
                    e
                ),
            },

            // File does not exist
            false => {
                log::error!(
                    "({:>4}/{:>4}) file not found: {:?}",
                    self.index + 1,
                    self.total_chapters,
                    raw_path.as_os_str().to_str()
                );
            }
        }
    }
}
