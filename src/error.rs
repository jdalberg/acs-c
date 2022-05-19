use crate::messages::MessageType;
use reqwest::StatusCode;
use std::{error::Error as StdError, fmt, io};

/// A `Result` alias where the `Err` case is `reqwest::Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Builder,
    Request,
    Redirect,
    Status(StatusCode),
    Body,
    Decode,
}

#[derive(Debug)]
pub(crate) struct TimedOut;

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("operation timed out")
    }
}
impl StdError for TimedOut {}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}
pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

struct Inner {
    kind: Kind,
    source: Option<BoxError>,
    message_type: Option<MessageType>,
}

/// The Errors that may occur when processing a `Request`.
///
/// Note: Errors may include the full URL used to make the `Request`. If the URL
/// contains sensitive information (e.g. an API key as a query parameter), be
/// sure to remove it ([`without_url`](Error::without_url))
pub struct Error {
    inner: Box<Inner>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.kind {
            Kind::Builder => f.write_str("builder error")?,
            Kind::Request => f.write_str("error sending request")?,
            Kind::Body => f.write_str("request or response body error")?,
            Kind::Decode => f.write_str("error decoding response body")?,
            Kind::Redirect => f.write_str("error following redirect")?,
            Kind::Status(ref code) => {
                let prefix = if code.is_client_error() {
                    "HTTP status client error"
                } else {
                    debug_assert!(code.is_server_error());
                    "HTTP status server error"
                };
                write!(f, "{} ({})", prefix, code)?;
            }
        };

        if let Some(mt) = &self.inner.message_type {
            write!(f, " for message_type ({})", mt.to_string())?;
        }

        if let Some(e) = &self.inner.source {
            write!(f, ": {}", e)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("acs-c::Error");

        builder.field("kind", &self.inner.kind);

        if let Some(ref mt) = self.inner.message_type {
            builder.field("message_type", mt);
        }
        if let Some(ref source) = self.inner.source {
            builder.field("source", source);
        }

        builder.finish()
    }
}

impl Error {
    pub(crate) fn new<E>(kind: Kind, source: Option<E>) -> Error
    where
        E: Into<BoxError>,
    {
        Error {
            inner: Box::new(Inner {
                kind,
                source: source.map(Into::into),
                message_type: None,
            }),
        }
    }

    /// Returns true if the error is from a type Builder.
    pub fn is_builder(&self) -> bool {
        matches!(self.inner.kind, Kind::Builder)
    }

    /// Returns true if the error is from a `RedirectPolicy`.
    pub fn is_redirect(&self) -> bool {
        matches!(self.inner.kind, Kind::Redirect)
    }

    /// Returns true if the error is from `Response::error_for_status`.
    pub fn is_status(&self) -> bool {
        matches!(self.inner.kind, Kind::Status(_))
    }

    /// Returns true if the error is related to a timeout.
    pub fn is_timeout(&self) -> bool {
        let mut source = self.source();

        while let Some(err) = source {
            if err.is::<TimedOut>() {
                return true;
            }
            source = err.source();
        }

        false
    }

    /// Returns true if the error is related to the request
    pub fn is_request(&self) -> bool {
        matches!(self.inner.kind, Kind::Request)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Returns true if the error is related to connect
    pub fn is_connect(&self) -> bool {
        let mut source = self.source();

        while let Some(err) = source {
            if let Some(hyper_err) = err.downcast_ref::<hyper::Error>() {
                if hyper_err.is_connect() {
                    return true;
                }
            }

            source = err.source();
        }

        false
    }

    /// Returns true if the error is related to the request or response body
    pub fn is_body(&self) -> bool {
        matches!(self.inner.kind, Kind::Body)
    }

    /// Returns true if the error is related to decoding the response's body
    pub fn is_decode(&self) -> bool {
        matches!(self.inner.kind, Kind::Decode)
    }

    /// Returns the status code, if the error was generated from a response.
    pub fn status(&self) -> Option<StatusCode> {
        match self.inner.kind {
            Kind::Status(code) => Some(code),
            _ => None,
        }
    }

    // private

    #[allow(unused)]
    pub(crate) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self)
    }
}

pub(crate) fn builder<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder, Some(e))
}

pub(crate) fn body<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Body, Some(e))
}

pub(crate) fn decode<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Decode, Some(e))
}

pub(crate) fn request<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request, Some(e))
}
