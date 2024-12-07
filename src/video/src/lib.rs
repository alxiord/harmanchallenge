use std::fmt::{self, Display};
use std::{rc::Rc, result::Result};

pub mod gst;

#[derive(Debug)]
pub enum Error {
    Gstreamer(gst::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Gstreamer(e) => write!(f, "Gstreamer error: {}", e),
        }
    }
}

pub trait Decoder {
    fn new() -> Result<Rc<Self>, Error>;
    fn build(&mut self) -> Result<(), Error>;
}
