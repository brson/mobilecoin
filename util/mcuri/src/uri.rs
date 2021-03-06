// Copyright (c) 2018-2020 MobileCoin Inc.

use crate::traits::{ConnectionUri, UriScheme};
use failure::Fail;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    str::FromStr,
};
use url::Url;

#[derive(Debug, Fail)]
pub enum UriParseError {
    #[fail(display = "Url parse error: {}", _0)]
    UrlParse(url::ParseError),

    #[fail(display = "Missing host")]
    MissingHost,

    #[fail(display = "Unknown scheme")]
    UnknownScheme,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uri<Scheme: UriScheme> {
    /// The original Url object used to construct this object.
    url: Url,

    /// Hostname.
    host: String,

    /// Consensus port.
    port: u16,

    /// Whether to use TLS when connecting.
    use_tls: bool,

    /// The uri scheme
    _scheme: PhantomData<fn() -> Scheme>,
}

impl<Scheme: UriScheme> ConnectionUri for Uri<Scheme> {
    fn url(&self) -> &Url {
        &self.url
    }

    fn host(&self) -> String {
        self.host.clone()
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn use_tls(&self) -> bool {
        self.use_tls
    }
}

impl<Scheme: UriScheme> Display for Uri<Scheme> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let scheme = if self.use_tls {
            Scheme::SCHEME_SECURE
        } else {
            Scheme::SCHEME_INSECURE
        };
        write!(f, "{}://{}:{}/", scheme, self.host, self.port)
    }
}

impl<Scheme: UriScheme> FromStr for Uri<Scheme> {
    type Err = UriParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(src).map_err(UriParseError::UrlParse)?;

        let host = url
            .host_str()
            .ok_or(UriParseError::MissingHost)?
            .to_string();
        if host.is_empty() {
            return Err(UriParseError::MissingHost);
        }

        let use_tls = if url.scheme().starts_with(Scheme::SCHEME_SECURE) {
            true
        } else if url.scheme().starts_with(Scheme::SCHEME_INSECURE) {
            false
        } else {
            return Err(UriParseError::UnknownScheme);
        };

        let port = match (url.port(), use_tls) {
            (Some(port), _) => port,
            (None, true) => Scheme::DEFAULT_SECURE_PORT,
            (None, false) => Scheme::DEFAULT_INSECURE_PORT,
        };

        Ok(Self {
            url,
            host,
            port,
            use_tls,
            _scheme: Default::default(),
        })
    }
}

impl<Scheme: UriScheme> serde::Serialize for Uri<Scheme> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.url.as_str())
    }
}

impl<'de, Scheme: UriScheme> serde::Deserialize<'de> for Uri<Scheme> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct UriVisitor<Scheme: UriScheme> {
            pub _scheme: PhantomData<Scheme>,
        }

        impl<'de, S: UriScheme> Visitor<'de> for UriVisitor<S> {
            type Value = Uri<S>;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("a string representing an URL")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Uri::from_str(s).map_err(|err| {
                    Error::invalid_value(Unexpected::Str(s), &err.to_string().as_str())
                })
            }
        }

        deserializer.deserialize_str(UriVisitor::<Scheme> {
            _scheme: Default::default(),
        })
    }
}
