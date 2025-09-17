use std::{fmt::Display, num::NonZeroU16, ops::Deref, str::FromStr};

use ::url::Url;

/// Url which is guaranteed to have a host, port and a supported scheme
#[derive(Clone, Debug)]
pub struct HostUrl(::url::Url);

impl HostUrl {
    pub fn host(&self) -> ::url::Host<&str> {
        self.0.host().expect("Checked during construction")
    }

    pub fn port(&self) -> NonZeroU16 {
        self.0
            .port()
            .and_then(NonZeroU16::new)
            .expect("Checked during construction")
    }
    pub fn set_port(&mut self, value: NonZeroU16) {
        self.0.set_port(Some(value.get())).unwrap()
    }
}

impl Display for HostUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for HostUrl {
    type Target = ::url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for HostUrl {
    type Err = HostUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<Url> for HostUrl {
    type Error = HostUrlError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        if !matches!(value.scheme(), "http" | "https") {
            Err(HostUrlError::UnknownScheme(value))
        } else if value.host().is_none() {
            Err(HostUrlError::MissingHost(value))
        } else {
            let raw_port = value.port();
            let mut r = HostUrl(value);

            match raw_port {
                Some(0) | None => {
                    r.set_port(const { NonZeroU16::new(11434).unwrap() });
                    Ok(r)
                }
                Some(_) => Ok(r),
            }
        }
    }
}

impl TryFrom<&str> for HostUrl {
    type Error = HostUrlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<::url::Url>()?.try_into()
    }
}

impl TryFrom<String> for HostUrl {
    type Error = HostUrlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&String> for HostUrl {
    type Error = HostUrlError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HostUrlError {
    #[error("{0}")]
    Parse(#[from] ::url::ParseError),
    #[error("Missing host: {0}")]
    MissingHost(::url::Url),
    #[error("Unknown scheme {0}")]
    UnknownScheme(::url::Url),
}
