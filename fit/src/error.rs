use failure::{Backtrace, Context, Fail};
use std::fmt;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Failed reading FIT file")]
    FileRead(std::io::ErrorKind),
    #[fail(display = "Unsupported: File contains developer fields")]
    HasDeveloperFields,
    #[fail(display = "Definition for msg {} could not be found", _0)]
    MissingDefinition(u8),
    #[fail(display = "Parsed value was not within expected range")]
    UnexpectedValue,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::FileRead(err.kind())),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}
