//! download command

use crate::{
    config::{Config, ProxyConfig},
    utils,
};
use clap::Args;
use std::{collections::VecDeque, fs::File, io::Write, path::Path, sync::Arc};
use tokio::sync::Mutex;
use wnrake::{
    book::UrlCache,
    client::Client,
    error::Error,
    parser::{Downloader, WnParser},
};

#[derive(Args, Clone, Debug)]
pub struct Download {
    /// Use multiple threads [one for each configured proxy]
    #[arg(short = 't', long)]
    use_threads: bool,
}

impl Download {
    pub async fn execute<'a>(&self, config: &Config) -> Result<(), Error> {
        if self.use_threads {
            let proxies = config
                .load_config_file()
                .proxies
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<ProxyConfig>>();
            let proxy_len = proxies.len();
            match proxy_len {
                0 => Err(Error::solver("must have at least 1 proxy configured")),
                1 => {
                    log::warn!("only 1 proxy found, falling back to single thread");
                    let mut client = config.to_client_with_proxy(proxies.first().unwrap());
                    self.single_thread(&mut client).await
                }
                _ => self.multi_thread(config, proxies).await,
            }
        } else {
            let mut client = config.to_client();
            self.single_thread(&mut client).await
        }
    }

    async fn single_thread(&self, client: &mut Client) -> Result<(), Error> {
        log::debug!("Solver={}", client.solver());
        log::debug!("Proxy={:?}", client.proxy());
        log::debug!("Cache={:?}", client.cache());

        client.create_session().await?;
        let res = async || -> Result<(), Error> {
            // Make staging directory
            utils::ensure_dir("staging")?;

            // Load URL cache
            log::debug!("loading urlcache.txt");
            let url_cache = UrlCache::from_file("urlcache.txt")?;
            let total_chapters = url_cache.as_ref().len();
            log::debug!("total chapters: {}", total_chapters);

            for (i, url) in url_cache.as_ref().iter().enumerate() {
                download_chapter(client, i, total_chapters, url).await?;
            }
            Ok(())
        }()
        .await;
        client.destroy_session().await?;
        res
    }

    async fn multi_thread(&self, config: &Config, proxies: Vec<ProxyConfig>) -> Result<(), Error> {
        // Make staging directory
        utils::ensure_dir("staging")?;

        // Load URL cache
        log::debug!("loading urlcache.txt");
        let url_cache = UrlCache::from_file("urlcache.txt")?;
        let total_chapters = url_cache.as_ref().len();
        log::debug!("total chapters: {}", total_chapters);

        // Build workers
        let url_cache = Arc::new(Mutex::new(
            url_cache.0.into_iter().enumerate().collect::<VecDeque<_>>(),
        ));
        let workers = proxies
            .iter()
            .map(|proxy| {
                log::debug!("{:?}", proxy);
                Worker {
                    client: config.to_client_with_proxy(proxy),
                    total_chapters: total_chapters,
                    urls: url_cache.clone(),
                }
            })
            .collect::<Vec<_>>();

        // Do work
        let futures = workers
            .into_iter()
            .map(|worker| tokio::spawn(worker.do_work()))
            .collect::<Vec<_>>();

        // Wait for work to complete
        for (i, future) in futures.into_iter().enumerate() {
            match future.await {
                Ok(_) => log::debug!("worker {} finished", i),
                Err(e) => log::warn!("worker {}: {}", i, e),
            }
        }

        // Check if all URLs were consumed
        let urls = url_cache.as_ref().lock().await;
        match urls.len() {
            0 => Ok(()),
            _ => Err(Error::solver("not all URLs were downloaded successfully")),
        }
    }
}

async fn download_chapter(
    client: &mut Client,
    i: usize,
    total_chapters: usize,
    url: &str,
) -> Result<(), Error> {
    // Get path
    let filename = utils::url_to_filename(i, url);
    let path = Path::join(Path::new("staging"), &filename);

    match path.is_file() {
        true => {
            log::info!("({:>4}/{:>4}) Using cached {}", i + 1, total_chapters, url);
        }
        false => {
            // Load parser
            let parser = WnParser::try_from(url)?;
            log::debug!("using parser {:?}", parser);

            // Download
            log::info!("({:>4}/{:>4}) Downloading {}", i + 1, total_chapters, url);
            let chapter = parser.get_chapter(client, url).await?;

            // Write file
            let mut file = File::create(path)?;
            file.write_all(chapter.as_bytes())?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Worker {
    client: Client,
    total_chapters: usize,
    urls: Arc<Mutex<VecDeque<(usize, String)>>>,
}

impl Worker {
    pub async fn do_work(mut self) {
        log::debug!("Solver={}", self.client.solver());
        log::debug!("Proxy={:?}", self.client.proxy());
        log::debug!("Cache={:?}", self.client.cache());
        match self.client.create_session().await {
            Ok(_) => loop {
                let task = {
                    let mut urls = self.urls.as_ref().lock().await;
                    urls.pop_front()
                };
                match task {
                    Some((i, url)) => {
                        match download_chapter(&mut self.client, i, self.total_chapters, &url).await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                log::warn!("worker: {}", e);
                                let mut urls = self.urls.as_ref().lock().await;
                                urls.push_front((i, url));
                                break;
                            }
                        }
                    }
                    None => break,
                }
            },
            Err(e) => log::error!("worker: {}", e),
        }
        let _ = self.client.destroy_session().await;
    }
}
