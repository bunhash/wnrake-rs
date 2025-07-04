//! Book structures

#[derive(Clone, Debug)]
pub struct BookInfo {
    pub title: String,
    pub author: String,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub title: String,
    pub html: String,
}
