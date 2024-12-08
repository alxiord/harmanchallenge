use std::cell::RefCell;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gstreamer::prelude::{
    Cast, ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, ObjectExt, PadExt,
};
use gstreamer::{glib, Element, ElementFactory, Pipeline};

use crate::DecoderOptions;

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

impl GstreamerDecoder {
    fn hardcode_mp4_input() -> Result<Vec<Element>, VideoError> {
        Ok(vec![
            ElementFactory::make("qtdemux")
                .name("demux")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ElementFactory::make("avdec_h264")
                .name("avdec_h264-0")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ElementFactory::make("videoconvert")
                .name("videoconvert0")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            //     ElementFactory::make("coloreffects")
            //         .name("coloreffects0")
            //         .property("preset", 3u32) // preset=3 as per your working pipeline
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            //     ElementFactory::make("videoflip")
            //         .name("videoflip0")
            //         .property("method", "horizontal-flip")
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            //     ElementFactory::make("videoscale")
            //         .name("videoscale0")
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            //     ElementFactory::make("capsfilter")
            //         .name("capsfilter0")
            //         .property(
            //             "caps",
            //             gstreamer::Caps::builder("video/x-raw")
            //                 .field("width", 600i32)
            //                 .field("height", 400i32)
            //                 .build(),
            //         )
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            //     ElementFactory::make("xvimagesink")
            //         .name("xvimagesink0")
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
        ])
    }

    fn handle_demux_pad_added(
        demux_src_pad: &gstreamer::Pad,
        next_elem: &gstreamer::Element, // decoder
    ) {
        let next_elem_sink_pad = next_elem.static_pad("sink").unwrap();
        if let Err(e) = demux_src_pad.link(&next_elem_sink_pad) {
            eprintln!("Failed to link demux pad to decoder: {}", e);
        } else {
            println!("Successfully linked demux pad to decoder.");
        }
    }
}

impl super::Decoder for GstreamerDecoder {
    fn new() -> Result<Arc<Mutex<Self>>, VideoError> {
        gstreamer::init().map_err(|e| VideoError::Gstreamer(Error::Glib(e)))?;

        let src: Element = ElementFactory::make("filesrc")
            .name("filesrc0")
            .property("location", "input/hello.mp4")
            .build()
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        let sink: Element = ElementFactory::make("xvimagesink")
            .name("xvimagesink0")
            .build()
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        let pipeline = Pipeline::with_name("hc-pipeline");

        Ok(Arc::new(Mutex::new(GstreamerDecoder {
            steps: vec![src, sink],
            pipeline,
        })))
    }

    fn build(self_rc: Arc<Mutex<Self>>, opts: DecoderOptions) -> Result<(), VideoError> {
        let mut lock = self_rc.lock();
        let decoder = lock.as_deref_mut().unwrap();

        let hardcoded_mp4_input = Self::hardcode_mp4_input()?;
        decoder.steps.splice(1..1, hardcoded_mp4_input);

        decoder
            .pipeline
            .add_many(decoder.steps.iter())
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        for i in 0..decoder.steps.len() - 1 {
            if decoder.steps[i].name() == "demux" {
                let next_elem = decoder.steps[i + 1].clone();

                // Use glib::clone! macro for cleaner syntax and handling
                decoder.steps[i].connect_pad_added(move |_demux, src_pad| {
                    // let self_clone = Rc::clone(&self_clone);
                    let next_elem = next_elem.clone();
                    GstreamerDecoder::handle_demux_pad_added(&src_pad, &next_elem);
                });

                decoder.steps[i]
                    .sync_state_with_parent()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;
            } else {
                decoder.steps[i]
                    .link(&decoder.steps[i + 1])
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;
            }
        }

        decoder
            .pipeline
            .set_state(gstreamer::State::Playing)
            .unwrap();

        let bus = decoder
            .pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    eprintln!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    decoder.pipeline.set_state(gstreamer::State::Null).unwrap();
                }
                _ => (),
            }
        }

        decoder.pipeline.set_state(gstreamer::State::Null).unwrap();

        Ok(())
    }
}
