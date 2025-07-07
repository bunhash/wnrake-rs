//! utils

use std::{fs, path::Path};
use wnrake::error::Error;

pub fn ensure_dir(dir: &str) -> Result<(), Error> {
    let dir = Path::new(dir);
    if !dir.is_dir() {
        log::debug!("creating {:?} directory", dir.as_os_str().to_str());
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub fn url_to_filename(index: usize, url: &str) -> String {
    let filename = url.rsplit("/").next().unwrap_or("chapter").replace(" ", "");
    format!("{:04}-{}", index, filename)
}

pub fn index_to_filename(index: usize) -> String {
    format!("{:04}.html", index + 1)
}
