use std::fmt::Debug;

#[cfg(feature = "macros")]
pub use ollama_rs_macros::function;
pub use url::*;

#[cfg(feature = "macros")]
pub mod re_exports {
    pub use schemars;
    pub use serde;
}

pub mod coordinator;
pub mod error;
pub mod generation;
#[cfg_attr(docsrs, doc(cfg(feature = "headers")))]
#[cfg(feature = "headers")]
pub mod headers;
pub mod history;
pub mod models;
mod url;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) url: HostUrl,
    pub(crate) reqwest_client: reqwest::Client,
    #[cfg(feature = "headers")]
    pub(crate) request_headers: reqwest::header::HeaderMap,
}

/// The main struct representing an Ollama client.
///
/// This struct is used to interact with the Ollama service.
///
/// # Fields
///
/// * `url` - The base URL of the Ollama service.
/// * `reqwest_client` - The HTTP client used for requests.
/// * `request_headers` - Optional headers for requests (enabled with the `headers` feature).
impl Ollama {
    /// Creates a new `Ollama` instance with the specified host and port.
    ///
    /// # Arguments
    ///
    /// * `host` - The host of the Ollama service.
    /// * `port` - The port of the Ollama service.
    ///
    /// # Returns
    ///
    /// A new `Ollama` instance.
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new<TUrl>(host: TUrl, port: u16) -> Self
    where
        TUrl: TryInto<HostUrl>,
        TUrl::Error: Debug,
    {
        let mut url: HostUrl = host.try_into().unwrap();
        url.set_port(Some(port)).unwrap();

        Self::from_url(url)
    }

    /// Creates a new `Ollama` instance with the specified host, port, and `reqwest` client.
    ///
    /// # Arguments
    ///
    /// * `host` - The host of the Ollama service.
    /// * `port` - The port of the Ollama service.
    /// * `reqwest_client` - The `reqwest` client instance.
    ///
    /// # Returns
    ///
    /// A new `Ollama` instance with the specified `reqwest` client.
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new_with_client<TUrl>(host: TUrl, port: u16, reqwest_client: reqwest::Client) -> Self
    where
        TUrl: TryInto<HostUrl>,
        TUrl::Error: Debug,
    {
        let mut url: HostUrl = host.try_into().unwrap();
        url.set_port(Some(port)).unwrap();

        Self::from_url_with_client(url, reqwest_client)
    }

    /// Attempts to create a new `Ollama` instance from a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the Ollama service.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Ollama` instance or a `url::ParseError`.
    #[inline]
    pub fn try_new<TUrl: TryInto<HostUrl>>(url: TUrl) -> Result<Self, TUrl::Error> {
        Ok(Self::from_url(url.try_into()?))
    }

    /// Create new instance from a [`Url`].
    #[inline]
    pub fn from_url(url: HostUrl) -> Self {
        Self::from_url_with_client(url, Default::default())
    }

    pub fn from_url_with_client(url: HostUrl, reqwest_client: reqwest::Client) -> Self {
        Self {
            url,
            reqwest_client,
            #[cfg(feature = "headers")]
            request_headers: reqwest::header::HeaderMap::new(),
        }
    }

    /// Returns the URI of the Ollama service as a `String`.
    ///
    /// # Panics
    ///
    /// Panics if the URL does not have a host.
    #[inline]
    pub fn host(&self) -> ::url::Host<&str> {
        self.url.host()
    }

    /// Returns a reference to the URL of the Ollama service.
    pub fn url(&self) -> &HostUrl {
        &self.url
    }

    /// Returns the URL of the Ollama service as a `&str`.
    ///
    /// Syntax in pseudo-BNF:
    ///
    /// ```bnf
    ///   url = scheme ":" [ hierarchical | non-hierarchical ] [ "?" query ]? [ "#" fragment ]?
    ///   non-hierarchical = non-hierarchical-path
    ///   non-hierarchical-path = /* Does not start with "/" */
    ///   hierarchical = authority? hierarchical-path
    ///   authority = "//" userinfo? host [ ":" port ]?
    ///   userinfo = username [ ":" password ]? "@"
    ///   hierarchical-path = [ "/" path-segment ]+
    /// ```
    #[inline]
    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }
}

impl From<HostUrl> for Ollama {
    fn from(url: HostUrl) -> Self {
        Self::from_url(url)
    }
}

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self::from_url("http://127.0.0.1:11434".parse().unwrap())
    }
}
