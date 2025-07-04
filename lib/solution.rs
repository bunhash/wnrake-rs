//! General Flaresolverr Response and Solution formats

use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Solution {
    pub url: String,
    pub status: u16,
    pub headers: Value,
    pub response: String,
    pub cookies: Value,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Response {
    pub status: String,
    pub message: String,
    #[serde(rename = "startTimestamp")]
    pub start_timestamp: u64,
    #[serde(rename = "endTimestamp")]
    pub end_timestamp: u64,
    pub version: String,
    pub session: Option<String>,
    pub sessions: Option<Vec<String>>,
    pub solution: Option<Solution>,
}
