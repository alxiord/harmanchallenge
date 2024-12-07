use std::fmt::{self, Display};
use std::io::Sink;
use std::rc::Rc;

use gstreamer::prelude::{ElementExtManual, GstBinExtManual};
use gstreamer::{glib, Element, ElementFactory, Pipeline};

use super::Error as VideoError;

#[derive(Debug)]
pub enum Error {
    Glib(glib::Error),
    GlibBool(glib::BoolError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Glib(e) => write!(f, "glib error: {}", e),
            Error::GlibBool(e) => write!(f, "glib bool error: {}", e),
        }
    }
}

pub struct GstreamerDecoder {
    steps: Vec<Element>,
    pipeline: Pipeline,
}

// cmd to successfully run the video w gstreamer:
// gst-launch-1.0 -v filesrc location=input/hello.mp4 ! qtdemux ! h264parse ! avdec_h264 ! videoconvert ! xvimagesink
// todo why do i need queue and videoconvert?

// to FLIP:
// ... videoflip method=horizontal-flip !  xvimagesink

impl super::Decoder for GstreamerDecoder {
    fn new() -> Result<Rc<Self>, VideoError> {
        gstreamer::init().map_err(|e| VideoError::Gstreamer(Error::Glib(e)))?;

        let src: Element = ElementFactory::make("filesrc")
            .name("hc-inputfsrc")
            .build()
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        let sink: Element = ElementFactory::make("xvimagesink")
            .name("hc=outimgsink")
            .build()
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        let pipeline = Pipeline::with_name("hc-pipeline");

        Ok(Rc::new(GstreamerDecoder {
            steps: [src, sink].to_vec(),
            pipeline,
        }))
    }

    fn build(&mut self) -> Result<(), VideoError> {
        self.pipeline
            .add_many(self.steps.iter())
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        for i in 0..self.steps.len() - 1 {
            self.steps[i]
                .link(&self.steps[i + 1])
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;
        }

        Ok(())
    }
}
