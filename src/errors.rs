use std::error::Error as StdError;
use std::fmt;
use std::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Error {
    Database,
    HttpExtern,
    JsonDecode,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::Database => "database error",
            Error::HttpExtern => "error communicating with external service",
            Error::JsonDecode => "error decoding JSON payload",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}
