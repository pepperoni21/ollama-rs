use crate::{IntoUrl, Ollama};

pub use http::header::*;

impl Ollama {
    /// Creates a new `Ollama` instance with the specified host, port, and request headers.
    ///
    /// # Arguments
    ///
    /// * `host` - The host of the Ollama service.
    /// * `port` - The port of the Ollama service.
    /// * `headers` - The request headers to be used.
    ///
    /// # Returns
    ///
    /// A new `Ollama` instance with the specified request headers.
    ///
    /// # Panics
    ///
    /// Panics if the host is not a valid URL or if the URL cannot have a port.
    pub fn new_with_request_headers(host: impl IntoUrl, port: u16, headers: HeaderMap) -> Self {
        let mut ollama = Self::new(host, port);
        ollama.set_headers(Some(headers));

        ollama
    }

    /// Sets the request headers for the `Ollama` instance.
    ///
    /// # Arguments
    ///
    /// * `headers` - An optional `HeaderMap` containing the request headers.
    ///
    /// If `None` is provided, the headers will be reset to an empty `HeaderMap`.
    pub fn set_headers(&mut self, headers: Option<HeaderMap>) {
        match headers {
            Some(h) => self.request_headers = h,
            None => self.request_headers = HeaderMap::new(),
        }
    }
}
