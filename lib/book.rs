//! Book structures

use crate::error::Error;
use std::{
    fs::File,
    io::{self, BufRead, Write},
};

#[cfg(windows)]
const LINE_ENDING: &'static [u8] = b"\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static [u8] = b"\n";

#[derive(Clone, Debug)]
pub struct BookInfo {
    pub title: String,
    pub author: String,
    pub url: String,
}

impl BookInfo {
    pub fn new() -> Self {
        BookInfo {
            title: String::new(),
            author: String::new(),
            url: String::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Error> {
        let mut title = String::new();
        let mut author = String::new();
        let mut url = String::new();

        let mut file = io::BufReader::new(File::open(path)?);
        file.read_line(&mut title)?;
        file.read_line(&mut author)?;
        file.read_line(&mut url)?;

        if title.is_empty() || author.is_empty() || url.is_empty() {
            Err(Error::parser("invalid bookinfo file"))
        } else {
            Ok(BookInfo {
                title: title.trim().to_string(),
                author: author.trim().to_string(),
                url: url.trim().to_string(),
            })
        }
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        file.write_all(self.title.as_bytes())?;
        file.write_all(LINE_ENDING)?;
        file.write_all(self.author.as_bytes())?;
        file.write_all(LINE_ENDING)?;
        file.write_all(self.url.as_bytes())?;
        file.write_all(LINE_ENDING)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct UrlCache(pub Vec<String>);

impl AsRef<Vec<String>> for UrlCache {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl AsMut<Vec<String>> for UrlCache {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

impl UrlCache {
    pub fn new() -> Self {
        UrlCache(Vec::new())
    }

    pub fn from_file(path: &str) -> Result<Self, Error> {
        let file = io::BufReader::new(File::open(path)?);
        let urls = file
            .lines()
            .filter_map(|line| {
                let line = line.ok()?.trim().to_string();
                if line.is_empty() { None } else { Some(line) }
            })
            .collect::<Vec<String>>();
        Ok(UrlCache(urls))
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        for url in &self.0 {
            file.write_all(url.as_bytes())?;
            file.write_all(LINE_ENDING)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub title: String,
    pub html: String,
}

#[derive(Clone, Debug)]
pub struct ChapterInfo {
    pub filename: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct ChapterList(pub Vec<ChapterInfo>);

impl AsRef<Vec<ChapterInfo>> for ChapterList {
    fn as_ref(&self) -> &Vec<ChapterInfo> {
        &self.0
    }
}

impl AsMut<Vec<ChapterInfo>> for ChapterList {
    fn as_mut(&mut self) -> &mut Vec<ChapterInfo> {
        &mut self.0
    }
}

impl ChapterList {
    pub fn new() -> Self {
        ChapterList(Vec::new())
    }

    pub fn from_file(path: &str) -> Result<Self, Error> {
        let mut chapterlist = ChapterList::new();
        let file = io::BufReader::new(File::open(path)?);
        for line in file.lines().filter_map(|line| {
            let line = line.ok()?.trim().to_string();
            if line.is_empty() { None } else { Some(line) }
        }) {
            let mut parts = line.split_ascii_whitespace();
            if parts.by_ref().count() != 2 {
                return Err(Error::parser("invalid chapterlist file"));
            }
            let filename = parts.next().expect("chapterinfo count should be 2").into();
            let title = parts.next().expect("chapterinfo count should be 2").into();
            chapterlist.0.push(ChapterInfo { filename, title });
        }
        Ok(chapterlist)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        for chapterinfo in &self.0 {
            file.write_all(chapterinfo.filename.as_bytes())?;
            file.write_all(b" ")?;
            file.write_all(chapterinfo.title.as_bytes())?;
            file.write_all(LINE_ENDING)?;
        }
        Ok(())
    }
}
