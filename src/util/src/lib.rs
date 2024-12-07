use std::error;
use std::fmt::{self, Display};
use std::io;

pub mod validator;

#[derive(Copy, Clone, Debug)]
pub enum VideoFormat {
    H264,
}

impl Display for VideoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoFormat::H264 => write!(f, "h264"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Format(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Format(e) => write!(f, "Invalid format: {}", e),
        }
    }
}

impl error::Error for Error {}
