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
/// use ollama_rs::IntoUrlSealed;
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
    /// Returns a new [`OllamaBuilder`] for fluently configuring an `Ollama`
    /// instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ollama_rs::Ollama;
    ///
    /// let ollama = Ollama::builder()
    ///     .host("http://localhost")
    ///     .port(11434)
    ///     .build();
    /// ```
    pub fn builder() -> OllamaBuilder {
        OllamaBuilder::new()
    }

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
    #[deprecated(
        since = "0.3.5",
        note = "use `Ollama::builder().host(host).port(port).build()` instead"
    )]
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
    #[deprecated(
        since = "0.3.5",
        note = "use `Ollama::builder().host(host).port(port).reqwest_client(client).build()` instead"
    )]
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

/// Builder for configuring and constructing an [`Ollama`] client.
///
/// Created via [`Ollama::builder`] or [`OllamaBuilder::new`]. All settings are
/// optional; the defaults match those of [`Ollama::default`] (host
/// `http://127.0.0.1:11434` and a freshly constructed [`reqwest::Client`]).
///
/// # Examples
///
/// ```
/// use ollama_rs::Ollama;
///
/// let ollama = Ollama::builder()
///     .host("http://localhost")
///     .port(11434)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct OllamaBuilder {
    url: Url,
    reqwest_client: Option<reqwest::Client>,
    #[cfg(feature = "headers")]
    request_headers: reqwest::header::HeaderMap,
}

impl OllamaBuilder {
    /// Creates a new builder pre-populated with the default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the host of the Ollama service.
    ///
    /// The host is parsed as a URL; the existing port (if any) is preserved
    /// unless overridden by [`OllamaBuilder::port`] or carried in `host`
    /// itself.
    ///
    /// # Panics
    ///
    /// Panics if `host` is not a valid URL.
    pub fn host(mut self, host: impl IntoUrl) -> Self {
        self.url = host.into_url().unwrap();
        self
    }

    /// Sets the port of the Ollama service.
    ///
    /// # Panics
    ///
    /// Panics if the URL cannot have a port.
    pub fn port(mut self, port: u16) -> Self {
        self.url.set_port(Some(port)).unwrap();
        self
    }

    /// Sets the full URL of the Ollama service, replacing any host or port
    /// previously configured.
    ///
    /// # Panics
    ///
    /// Panics if `url` is not a valid URL.
    pub fn url(mut self, url: impl IntoUrl) -> Self {
        self.url = url.into_url().unwrap();
        self
    }

    /// Sets a pre-configured [`reqwest::Client`] used to make requests to the
    /// Ollama service.
    pub fn reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }

    /// Sets the request headers attached to every request.
    #[cfg_attr(docsrs, doc(cfg(feature = "headers")))]
    #[cfg(feature = "headers")]
    pub fn request_headers(mut self, request_headers: reqwest::header::HeaderMap) -> Self {
        self.request_headers = request_headers;
        self
    }

    /// Consumes the builder and returns a configured [`Ollama`] instance.
    pub fn build(self) -> Ollama {
        Ollama {
            url: self.url,
            reqwest_client: self.reqwest_client.unwrap_or_default(),
            #[cfg(feature = "headers")]
            request_headers: self.request_headers,
        }
    }
}

impl Default for OllamaBuilder {
    /// Returns a builder pre-populated with the default Ollama host
    /// (`http://127.0.0.1:11434`).
    fn default() -> Self {
        Self {
            url: Url::parse("http://127.0.0.1:11434").unwrap(),
            reqwest_client: None,
            #[cfg(feature = "headers")]
            request_headers: reqwest::header::HeaderMap::new(),
        }
    }
}
