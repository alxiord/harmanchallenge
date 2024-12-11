#![deny(missing_docs)]

//! # Harman coding challenge - video processor
//!
//! ## Usage
//!
//! ```bash
//! cargo run -- --input=$INFILE [--width=$W] [--height=$H] [--format=$FORMAT] [--flip] [--invert]
//! ```
//!
//! ## Example
//!
//! To build and run a pipeline that opens an mp4 file, resizes it to 640x480, flips it
//! horizontally and inverts the colors, run:
//!
//! ```bash
//! cargo run -- --input=input/hello.mp4 --width=640 --height=480 --format=h264 --flip --invert
//! ```
//!
//! This is the equivalent of the following [`gstreamer`](https://gstreamer.freedesktop.org/) pipeline:
//!
//! ```bash
//! gst-launch-1.0 filesrc location=input/hello.mp4 !   \
//!     qtdemux name=demux demux.video_0 !              \
//!     avdec_h264 ! videoconvert !                     \
//!     coloreffects preset=3 ! videoconvert !          \
//!     videoscale ! video/x-raw,width=600,height=400 ! \
//!     videoflip method=horizontal-flip !              \
//!     xvimagesink
//! ```
//!
//! ## Links
//!
//! See also:
//! * the [`README`](https://github.com/alxiord/harmanchallenge/blob/main/README.md)
//! * the [`video`] documentation

use std::borrow::Borrow;

use util::{Cli, DecoderOptions};
use video::{
    gst::{self, GstreamerDecoder},
    Decoder, VideoInput,
};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let opts: DecoderOptions = cli.borrow().into();

    let insrc: VideoInput = match cli.input {
        Some(path) => VideoInput::File(path.as_path().to_string_lossy().to_string()),
        None => VideoInput::Webcam,
    };

    // Why an Arc<Mutex> when we can't see any threads?
    // Because Rust is paranoid.
    // Somewhere in ::build, a closure is needed because the demuxer component can only be
    // linked to the next element at "runtime", i.e. when the pipeline starts playing.
    // So we register a callback for that, which implies a closure, which from Rust's point of
    // view can be executed on any other thread, and supersede the decoder instance's lifetime too.
    // Conceptually this scenario makes no sense but I can't defeat the compiler sooo, Arc<Mutex>
    // to enforce thread safety and avoid lifetime headaches
    let decoder_mutex = gst::GstreamerDecoder::new(insrc).unwrap();

    GstreamerDecoder::build(decoder_mutex.clone(), opts).unwrap();

    let mut lock = decoder_mutex.lock();
    let decoder = lock.as_deref_mut().unwrap();
    decoder.run().unwrap();
}
