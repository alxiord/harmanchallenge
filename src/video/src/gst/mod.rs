use std::fmt::{self, Display};
use std::rc::Rc;

use gstreamer::{glib, Element, Pipeline};

use super::Error as VideoError;

#[derive(Debug)]
pub enum Error {
    Glib(glib::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Glib(e) => write!(f, "glib error: {}", e),
        }
    }
}

pub struct GstreamerDecoder {
    src: Option<Element>,
    sink: Option<Element>,
    pipeline: Option<Pipeline>,
}

impl super::Decoder for GstreamerDecoder {
    fn new() -> Result<Rc<Self>, VideoError> {
        gstreamer::init().map_err(|e| VideoError::Gstreamer(Error::Glib(e)))?;

        Ok(Rc::new(GstreamerDecoder {
            src: None,
            sink: None,
            pipeline: None,
        }))
    }
}
