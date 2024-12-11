#![crate_name = "video"]
// #![deny(missing_docs)]

//! Video manipulation and decoding utilities.
//! This crate exposes functionality for decoding a video file,
//! applying optional filters and outputting the result to the
//! screen.
//!
//! Support matrix:
//! * input: mp4 file
//! * output: h264-encoded, to screen
//! * filters:
//!   * resize to specified witdth x height
//!   * invert colors
//!   * horizontal flip
//!
//! Under the hood, the crate uses [`gstreamer`](https://gstreamer.freedesktop.org/).

use std::fmt::{self, Display};
use std::result::Result;
use std::sync::{Arc, Mutex};

use util::DecoderOptions;

/// Gstreamer based implementation
pub mod gst;

#[derive(Debug)]
/// Errors that can occur during video manipulation
pub enum Error {
    /// Gstreamer error
    Gstreamer(gst::Error),
    /// Mutex poisoned
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

pub enum VideoInput {
    File(String),
    Webcam,
}

/// Trait that defines the common interface for supported video manipulator structs
pub trait Decoder {
    /// Create a new instance
    fn new(input: VideoInput) -> Result<Arc<Mutex<Self>>, Error>;
    /// Add decoders, encoders and filters
    fn build(self_rc: Arc<Mutex<Self>>, opts: DecoderOptions) -> Result<(), Error>;
    /// Parse the input file and output the result to the screen
    fn run(&mut self) -> Result<(), Error>;
}
