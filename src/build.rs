//! build command

use clap::Args;
use std::{fs::File, path::Path, process::Command};
use wnrake::{
    book::{BookInfo, ChapterList, EpubBook},
    config::Config,
    error::Error,
};

#[derive(Args, Clone, Debug)]
pub struct Build {
    /// Appends `(Ongoing)` to the title
    #[arg(long)]
    ongoing: bool,

    /// Skip AZW3 conversion
    #[arg(long)]
    azw3: bool,
}

impl Build {
    pub fn execute<'a>(&self, _config: &Config) -> Result<(), Error> {
        let mut bookinfo = BookInfo::from_file("bookinfo.txt")?;
        let chapterlist = ChapterList::from_file("chapterlist.txt")?;
        let cover = {
            let path = Path::new("cover.jpg");
            match path.is_file() {
                true => Some(File::open(path)?),
                false => None,
            }
        };

        log::info!("Title: {}", bookinfo.title);
        log::info!("Author: {}", bookinfo.author);
        log::info!("Chapters: {}", chapterlist.as_ref().len());
        if cover.is_none() {
            log::warn!("No cover found");
        }
        log::info!("Building epub ...");

        if self.ongoing {
            bookinfo.title = format!("{} (Ongoing)", bookinfo.title.as_str());
        }

        let filename = format!("{}.epub", bookinfo.title.as_str());
        let epub = EpubBook::new(bookinfo, chapterlist, cover);
        epub.to_file(filename.as_str())?;

        if self.azw3 {
            log::info!("Converting to AZW3 ...");
            let azw3_filename = format!("{}.azw3", filename.trim_end_matches(".epub"));
            match Command::new("ebook-convert")
                .args([filename.as_str(), azw3_filename.as_str(), "--no-inline-toc"])
                .output()
            {
                Ok(_) => log::info!("Complete"),
                Err(e) => log::error!("{}", e),
            }
        } else {
            log::info!("Complete");
        }

        Ok(())
    }
}
