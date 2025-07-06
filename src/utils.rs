//! utils

use std::{fs, path::Path};
use wnrake::error::Error;

pub fn ensure_staging() -> Result<(), Error> {
    let dir = Path::new("staging");
    if !dir.is_dir() {
        log::debug!("creating staging directory");
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub fn to_filename(index: usize, url: &str) -> String {
    let filename = url.rsplit("/").next().unwrap_or("chapter").replace(" ", "");
    format!("{:04}-{}", index, filename)
}
