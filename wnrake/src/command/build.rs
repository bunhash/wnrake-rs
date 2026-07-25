//! build command

use crate::{
    book::{BookInfo, ChapterList, EpubBook},
    error::Error,
};
use clap::Args;
use crawler::config::Config;
use std::{
    fs::{File, remove_file, rename},
    path::Path,
    process::Command,
};

#[derive(Args, Clone, Debug)]
pub struct Build {
    /// Appends `(Ongoing)` to the title
    #[arg(long)]
    ongoing: bool,

    /// Appends `(Hiatus)` to the title
    #[arg(long)]
    hiatus: bool,

    /// Fixes some EPUB issues
    #[arg(long)]
    epub: bool,

    /// Do AZW3 conversion
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

        if self.hiatus {
            bookinfo.title = format!("{} (Hiatus)", bookinfo.title.as_str());
        } else if self.ongoing {
            bookinfo.title = format!("{} (Ongoing)", bookinfo.title.as_str());
        }

        let filename = format!("{}.epub", bookinfo.title.as_str());
        let epub = EpubBook::new(bookinfo, chapterlist, cover);
        epub.to_file(filename.as_str())?;

        if self.epub {
            log::info!("Converting to EPUB ...");
            let tmp_filename = format!("{} (tmp).epub", filename.trim_end_matches(".epub"));
            match Command::new("ebook-convert")
                .args([filename.as_str(), tmp_filename.as_str()])
                .output()
            {
                Ok(_) => match remove_file(filename.as_str()) {
                    Ok(_) => {
                        if let Err(e) = rename(tmp_filename.as_str(), filename.as_str()) {
                            log::error!("{}", e);
                        }
                    }
                    Err(e) => log::error!("{}", e),
                },
                Err(e) => log::error!("{}", e),
            }
        }

        if self.azw3 {
            log::info!("Converting to AZW3 ...");
            let azw3_filename = format!("{}.azw3", filename.trim_end_matches(".epub"));
            if let Err(e) = Command::new("ebook-convert")
                .args([filename.as_str(), azw3_filename.as_str(), "--no-inline-toc"])
                .output()
            {
                log::error!("{}", e);
            }
        }

        log::info!("Complete");
        Ok(())
    }
}
