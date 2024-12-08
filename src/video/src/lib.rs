use std::fmt::{self, Display};
use std::result::Result;
use std::sync::{Arc, Mutex};

pub mod gst;

#[derive(Debug)]
pub enum Error {
    Gstreamer(gst::Error),
    PoisonedLock,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Gstreamer(e) => write!(f, "Gstreamer error: {}", e),
            Error::PoisonedLock => write!(f, "Mutex poisoned"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DecoderOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub invert: bool,
    pub flip: bool,
}

impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            invert: false,
            flip: false,
        }
    }
}

pub trait Decoder {
    fn new(infname: &str) -> Result<Arc<Mutex<Self>>, Error>;
    fn build(self_rc: Arc<Mutex<Self>>, opts: DecoderOptions) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
}
