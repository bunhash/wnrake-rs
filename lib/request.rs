//! Flaresolverr Request Builder

use crate::proxy::Proxy;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "waitType", content = "waitFor")]
pub enum WaitFor {
    #[serde(rename = "id")]
    Id(String),

    #[serde(rename = "xpath")]
    XPath(String),

    #[serde(rename = "link")]
    Link(String),

    #[serde(rename = "exact-link")]
    ExactLink(String),

    #[serde(rename = "name")]
    Name(String),

    #[serde(rename = "tag")]
    Tag(String),

    #[serde(rename = "class")]
    Class(String),

    #[serde(rename = "selector")]
    Selector(String),
}

impl WaitFor {
    pub fn id(val: &str) -> Self {
        WaitFor::Id(val.into())
    }

    pub fn xpath(val: &str) -> Self {
        WaitFor::XPath(val.into())
    }

    pub fn link(val: &str) -> Self {
        WaitFor::Link(val.into())
    }

    pub fn exact_link(val: &str) -> Self {
        WaitFor::ExactLink(val.into())
    }

    pub fn name(val: &str) -> Self {
        WaitFor::Name(val.into())
    }

    pub fn tag(val: &str) -> Self {
        WaitFor::Tag(val.into())
    }

    pub fn class(val: &str) -> Self {
        WaitFor::Class(val.into())
    }

    pub fn selector(val: &str) -> Self {
        WaitFor::Selector(val.into())
    }

    /// Returns a reference to the type name
    pub fn type_name(&self) -> &str {
        match self {
            WaitFor::Id(_) => "id",
            WaitFor::XPath(_) => "xpath",
            WaitFor::Link(_) => "link",
            WaitFor::ExactLink(_) => "exact-link",
            WaitFor::Name(_) => "name",
            WaitFor::Tag(_) => "tag",
            WaitFor::Class(_) => "class",
            WaitFor::Selector(_) => "selector",
        }
    }

    /// Returns a reference to the value
    pub fn value(&self) -> &str {
        match self {
            WaitFor::Id(v) => v.as_str(),
            WaitFor::XPath(v) => v.as_str(),
            WaitFor::Link(v) => v.as_str(),
            WaitFor::ExactLink(v) => v.as_str(),
            WaitFor::Name(v) => v.as_str(),
            WaitFor::Tag(v) => v.as_str(),
            WaitFor::Class(v) => v.as_str(),
            WaitFor::Selector(v) => v.as_str(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Session<'a> {
    pub(crate) cmd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) proxy: Option<&'a Proxy>,
}

impl<'a> Session<'a> {
    pub fn create(proxy: Option<&'a Proxy>) -> Self {
        Session {
            cmd: "sessions.create".into(),
            session: None,
            proxy,
        }
    }

    pub fn destroy(session: &str) -> Self {
        Session {
            cmd: "sessions.destroy".into(),
            session: Some(session.into()),
            proxy: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Cookie {
    name: String,
    value: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Request {
    pub(crate) cmd: String,
    pub(crate) url: String,
    #[serde(rename = "maxTimeout")]
    pub(crate) max_timeout: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) session: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub(crate) wait_for: Option<WaitFor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cookies: Option<Vec<Cookie>>,
    #[serde(rename = "noKill")]
    pub(crate) no_kill: bool,
    #[serde(rename = "postData", skip_serializing_if = "Option::is_none")]
    pub(crate) post_data: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) enable_cache: bool,
}

impl Request {
    pub fn get(url: &str) -> RequestBuilder {
        RequestBuilder::get(url)
    }

    pub fn post(url: &str) -> RequestBuilder {
        RequestBuilder::post(url)
    }
}

#[derive(Clone, Debug)]
pub struct RequestBuilder(Request);

impl RequestBuilder {
    pub fn get(url: &str) -> RequestBuilder {
        RequestBuilder(Request {
            cmd: "request.get".into(),
            url: url.into(),
            max_timeout: 60000,
            session: None,
            wait_for: None,
            cookies: None,
            no_kill: true,
            post_data: None,
            enable_cache: true,
        })
    }

    pub fn post(url: &str) -> RequestBuilder {
        RequestBuilder(Request {
            cmd: "request.post".into(),
            url: url.into(),
            max_timeout: 60000,
            session: None,
            wait_for: None,
            cookies: None,
            no_kill: true,
            post_data: Some("".into()),
            enable_cache: false,
        })
    }

    pub fn wait_for(mut self, wait_for: WaitFor) -> Self {
        self.0.wait_for = Some(wait_for);
        self
    }

    pub fn cookies(mut self, cookies: &[(&str, &str)]) -> Self {
        self.0.cookies = Some(
            cookies
                .iter()
                .map(|(n, v)| Cookie {
                    name: n.to_string(),
                    value: v.to_string(),
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn with_kill(mut self) -> Self {
        self.0.no_kill = false;
        self
    }

    pub fn without_kill(mut self) -> Self {
        self.0.no_kill = true;
        self
    }

    pub fn post_data(mut self, post_data: &[(&str, &str)]) -> Self {
        let mut form = form_urlencoded::Serializer::new(String::new());
        for (k, v) in post_data {
            form.append_pair(k, v);
        }
        self.0.post_data = Some(form.finish());
        self
    }

    pub fn enable_cache(mut self) -> Self {
        self.0.enable_cache = true;
        self
    }

    pub fn disable_cache(mut self) -> Self {
        self.0.enable_cache = false;
        self
    }

    pub fn build(self) -> Request {
        self.0
    }
}
