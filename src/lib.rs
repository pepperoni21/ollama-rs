pub mod error;
pub mod generation;
#[cfg(feature = "chat-history")]
pub mod history;
pub mod models;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) url: url::Url,
    pub(crate) reqwest_client: reqwest::Client,
    #[cfg(feature = "chat-history")]
    pub(crate) messages_history: Option<history::MessagesHistory>,
}

impl Ollama {
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new(host: impl Into<url::Url>, port: u16) -> Self {
        let mut url = host.into();
        url.set_port(Some(port)).unwrap();

        Self::from_url(url)
    }

    /// Create new instance from a [`url::Url`].
    #[inline]
    pub fn from_url(url: url::Url) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    /// Returns the http URI of the Ollama instance
    ///
    /// # Panics
    ///
    /// Panics if the URL does not have a host.
    #[inline]
    pub fn uri(&self) -> String {
        self.url.host().unwrap().to_string()
    }

    /// Returns the URL of the Ollama instance as a [`url::Url`].
    pub fn url(&self) -> &url::Url {
        &self.url
    }

    /// Returns the URL of the Ollama instance as a [str].
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

impl From<url::Url> for Ollama {
    fn from(url: url::Url) -> Self {
        Self::from_url(url)
    }
}

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self {
            url: url::Url::parse("http://127.0.0.1:11434").unwrap(),
            reqwest_client: reqwest::Client::new(),
            #[cfg(feature = "chat-history")]
            messages_history: None,
        }
    }
}
