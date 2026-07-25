//! crawler

mod client;
mod error;
mod request;
mod response;

pub use client::{Client, ClientBuilder};
pub use error::{Error, ErrorType};
pub use request::{Cookie, Request, RequestBuilder, WaitFor};
pub use reqwest::Url;
pub use response::Solution;

pub mod proxy;

#[cfg(feature = "cache")]
pub mod cache;

#[cfg(feature = "config")]
pub mod config;
