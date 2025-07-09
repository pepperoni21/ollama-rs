#![cfg_attr(docsrs, feature(doc_cfg))]

use url::Url;

#[cfg(feature = "macros")]
pub use ollama_rs_macros::function;

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

/// A trait to try to convert some type into a [`Url`].
///
/// This trait is "sealed", such that only types within ollama-rs can
/// implement it.
///
/// # Examples
///
/// ```
/// use url::Url;
/// use ollama_rs::IntoUrl;
///
/// let url: Url = "http://example.com".into_url().unwrap();
/// ```
pub trait IntoUrl: IntoUrlSealed {}

impl IntoUrl for Url {}
impl IntoUrl for String {}
impl IntoUrl for &str {}
impl IntoUrl for &String {}

pub trait IntoUrlSealed {
    fn into_url(self) -> Result<Url, url::ParseError>;

    fn as_str(&self) -> &str;
}

impl IntoUrlSealed for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl IntoUrlSealed for &str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)?.into_url()
    }

    fn as_str(&self) -> &str {
        self
    }
}

impl IntoUrlSealed for &String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        (&**self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        (&*self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) url: Url,
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
    pub fn new(host: impl IntoUrl, port: u16) -> Self {
        let mut url: Url = host.into_url().unwrap();
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
    pub fn new_with_client(host: impl IntoUrl, port: u16, reqwest_client: reqwest::Client) -> Self {
        let mut url: Url = host.into_url().unwrap();
        url.set_port(Some(port)).unwrap();

        Self {
            url,
            reqwest_client,
            #[cfg(feature = "headers")]
            request_headers: reqwest::header::HeaderMap::new(),
        }
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
    pub fn try_new(url: impl IntoUrl) -> Result<Self, url::ParseError> {
        Ok(Self::from_url(url.into_url()?))
    }

    /// Create new instance from a [`Url`].
    #[inline]
    pub fn from_url(url: Url) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    /// Returns the URI of the Ollama service as a `String`.
    ///
    /// # Panics
    ///
    /// Panics if the URL does not have a host.
    #[inline]
    pub fn uri(&self) -> String {
        self.url.host().unwrap().to_string()
    }

    /// Returns a reference to the URL of the Ollama service.
    pub fn url(&self) -> &Url {
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

impl From<Url> for Ollama {
    fn from(url: Url) -> Self {
        Self::from_url(url)
    }
}

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self {
            url: Url::parse("http://127.0.0.1:11434").unwrap(),
            reqwest_client: reqwest::Client::new(),
            #[cfg(feature = "headers")]
            request_headers: reqwest::header::HeaderMap::new(),
        }
    }
}
