use std::{fmt::Display, ops::Deref, str::FromStr};

use ::url::Url;

/// Url which is guaranteed to have a Host part
#[derive(Clone, Debug)]
pub struct HostUrl(::url::Url);

impl HostUrl {
    pub fn host(&self) -> ::url::Host<&str> {
        self.0.host().expect("Checked during construction")
    }
    #[allow(clippy::result_unit_err)]
    pub fn set_port(&mut self, value: Option<u16>) -> Result<(), ()> {
        self.0.set_port(value)
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
        if value.host().is_some() {
            Ok(HostUrl(value))
        } else {
            Err(HostUrlError::MissingHost(value))
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
    #[error("Missing url: {0}")]
    MissingHost(::url::Url),
}
