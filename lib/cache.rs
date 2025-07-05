//! Cache handler

use crate::error::Error;
use chrono::Local;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Cache {
    dir: PathBuf,
    prefix: String,
}

impl Cache {
    /// Creates a new cache handler
    pub fn new(dir: &str) -> Result<Self, Error> {
        let cache = Cache {
            dir: dir.into(),
            prefix: Local::now().date_naive().to_string(),
        };
        cache.ensure_path()?;
        cache.clean()?;
        Ok(cache)
    }

    fn ensure_path(&self) -> Result<(), Error> {
        if !self.dir.is_dir() {
            fs::create_dir_all(&self.dir)?;
        }
        Ok(())
    }

    fn clean(&self) -> Result<(), Error> {
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?.path();
            if let Some(filename) = entry.file_name() {
                if !filename
                    .to_str()
                    .ok_or(Error::io("invalid file"))?
                    .starts_with(self.prefix.as_str())
                {
                    fs::remove_file(entry)?;
                }
            }
        }
        Ok(())
    }

    fn url_to_path(&self, url: &str) -> PathBuf {
        Path::join(
            &self.dir,
            &format!(
                "{}_{}",
                &self.prefix,
                url.replace(":", "").replace("//", "/").replace("/", "_")
            ),
        )
        .into()
    }

    /// Fetches from the cache
    pub fn get(&self, url: &str) -> Result<Option<String>, Error> {
        let path = self.url_to_path(url);
        if path.is_file() {
            Ok(Some(fs::read_to_string(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Inserts into the cache
    pub fn insert(&self, url: &str, data: &[u8]) -> Result<(), Error> {
        let path = self.url_to_path(url);
        let mut file = fs::File::create(&path)?;
        file.write_all(data)?;
        Ok(())
    }
}
