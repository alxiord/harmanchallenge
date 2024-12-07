use std::error;
use std::fmt::{self, Display};
use std::io;

pub mod validator;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {}
