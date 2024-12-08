use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, PadExt};
use gstreamer::{glib, Element, ElementFactory, Pipeline};

use crate::DecoderOptions;

use super::Error as VideoError;

#[derive(Debug)]
pub enum Error {
    Glib(glib::Error),
    GlibBool(glib::BoolError),
    PipelineStateChange(gstreamer::StateChangeError),
    Bus,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Glib(e) => write!(f, "glib error: {}", e),
            Error::GlibBool(e) => write!(f, "glib bool error: {}", e),
            Error::PipelineStateChange(e) => write!(f, "pipeline state change error: {}", e),
            Error::Bus => write!(f, "pipeline without bus"),
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
            // ElementFactory::make("coloreffects")
            //     .name("coloreffects0")
            //     .property("preset", 3u32) // preset=3 as per your working pipeline
            //     .build()
            //     .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            // ElementFactory::make("videoflip")
            //     .name("videoflip0")
            //     .property("method", "horizontal-flip")
            //     .build()
            //     .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,

            //     ElementFactory::make("xvimagesink")
            //         .name("xvimagesink0")
            //         .build()
            //         .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
        ])
    }

    fn change_res(opt_w_h: Option<(i32, i32)>) -> Result<Vec<Element>, VideoError> {
        if let Some((w, h)) = opt_w_h {
            return Ok(vec![
                ElementFactory::make("videoscale")
                    .name("videoscale0")
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
                ElementFactory::make("capsfilter")
                    .name("capsfilter0")
                    .property(
                        "caps",
                        gstreamer::Caps::builder("video/x-raw")
                            .field("width", w)
                            .field("height", h)
                            .build(),
                    )
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ]);
        }
        Ok(vec![])
    }

    fn handle_demux_pad_added(
        demux_src_pad: &gstreamer::Pad,
        next_elem: &gstreamer::Element, // decoder
    ) {
        // Sadly unwrap here, if the demuxer can't be linked to the next element,
        // the pipeline is broken
        let next_elem_sink_pad = next_elem
            .static_pad("sink")
            .expect("Can't create sink pad for demuxer");
        demux_src_pad
            .link(&next_elem_sink_pad)
            .expect("Can't link demuxer to next element");
    }
}

impl super::Decoder for GstreamerDecoder {
    fn new(infname: &str) -> Result<Arc<Mutex<Self>>, VideoError> {
        gstreamer::init().map_err(|e| VideoError::Gstreamer(Error::Glib(e)))?;

        let src: Element = ElementFactory::make("filesrc")
            .name("filesrc0")
            .property("location", infname)
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
        let decoder = lock.as_deref_mut().map_err(|_| VideoError::PoisonedLock)?;

        let extra_steps = Self::hardcode_mp4_input().and_then(|mut v| {
            let change_res_steps = Self::change_res(opts.width_height)?;
            v.extend(change_res_steps);
            Ok(v)
        })?;

        decoder.steps.splice(1..1, extra_steps);

        decoder
            .pipeline
            .add_many(decoder.steps.iter())
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        for i in 0..decoder.steps.len() - 1 {
            if decoder.steps[i].name() == "demux" {
                // Special handling for demux!!
                // Why?
                // Because as the name suggests it *demultiplexes* src into multiple streams,
                // and the next element can't know what to link to unless explicitly shown.
                //
                // inspo:
                // https://stackoverflow.com/a/65591800
                // https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/0b1be1178918166a2e519d82f2935d68034ad046/examples/src/bin/transmux.rs
                let next_elem = decoder.steps[i + 1].clone();

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

        Ok(())
    }

    fn run(&mut self) -> Result<(), VideoError> {
        self.pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| VideoError::Gstreamer(Error::PipelineStateChange(e)))?;

        let bus = self
            .pipeline
            .bus()
            .ok_or(VideoError::Gstreamer(Error::Bus))?;

        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(_) => {
                    self.pipeline
                        .set_state(gstreamer::State::Null)
                        .map_err(|e| VideoError::Gstreamer(Error::PipelineStateChange(e)))?;
                    // todo log error
                    break;
                }
                _ => (),
            }
        }

        self.pipeline
            .set_state(gstreamer::State::Null)
            .map_err(|e| VideoError::Gstreamer(Error::PipelineStateChange(e)))
            .map(|_| ())
    }
}
