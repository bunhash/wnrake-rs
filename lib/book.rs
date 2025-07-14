//! Book structures

use crate::error::Error;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
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
    pub path: String,
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
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return Err(Error::parser("invalid chapterlist file"));
            }
            let path = parts[0].into();
            let title = parts[1].into();
            chapterlist.0.push(ChapterInfo { path, title });
        }
        Ok(chapterlist)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        for chapterinfo in &self.0 {
            file.write_all(chapterinfo.path.as_bytes())?;
            file.write_all(b" ")?;
            file.write_all(chapterinfo.title.as_bytes())?;
            file.write_all(LINE_ENDING)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct EpubBook {
    bookinfo: BookInfo,
    chapterlist: ChapterList,
    cover: Option<File>,
}

static CSS_STYLE: &str = r#"
@namespace epub "http://www.idpf.org/2007/ops";
body {
    font-family: Cambria, Liberation Serif, Bitstream Vera Serif, Georgia, Times, Times New Roman, serif;
}
h2 {
    text-align: left;
    text-transform: uppercase;
    font-weight: 200;
}
ol {
    list-style-type: none;
}
ol > li:first-child {
    margin-top: 0.3em;
}
    nav[epub|type~='toc'] > ol > li > ol  {
    list-style-type:square;
}
    nav[epub|type~='toc'] > ol > li > ol > li {
    margin-top: 0.3em;
}
"#;

impl EpubBook {
    pub fn new(bookinfo: BookInfo, chapterlist: ChapterList, cover: Option<File>) -> Self {
        EpubBook {
            bookinfo,
            chapterlist,
            cover,
        }
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        let mut output: Vec<u8> = Vec::new();

        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        builder
            .metadata("title", self.bookinfo.title.as_str())?
            .metadata("author", self.bookinfo.author.as_str())?
            .stylesheet(CSS_STYLE.as_bytes())?;

        if let Some(cover) = &self.cover {
            builder.add_cover_image("cover.jpg", cover, "image/jpeg")?;
            builder.add_content(
                EpubContent::new(
                    "cover.xhtml",
                    r#"<html><image src="cover.jpg"/></html>"#.as_bytes(),
                )
                .title("Cover")
                .reftype(ReferenceType::Cover),
            )?;
        }

        let title_contents = format!(
            r#"<html><body><h1>{}</h1><h2>{}</h2></body></html>"#,
            self.bookinfo.title, self.bookinfo.author
        );
        builder.add_content(
            EpubContent::new("title.xhtml", title_contents.as_bytes())
                .title("Title")
                .reftype(ReferenceType::TitlePage),
        )?;

        builder.inline_toc();

        for chapter in self.chapterlist.as_ref() {
            let contents = File::open(chapter.path.as_str())?;
            let content_path = format!("{}.xhtml", chapter.path.trim_end_matches(".html"));
            builder.add_content(
                EpubContent::new(content_path.as_str(), contents).title(chapter.title.as_str()),
            )?;
        }

        builder.generate(&mut output)?;

        file.write_all(output.as_ref())?;

        Ok(())
    }
}
