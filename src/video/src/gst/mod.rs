use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

use glib::object::ObjectExt;
use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, PadExt};
use gstreamer::prelude::{GObjectExtManualGst, ObjectType};
use gstreamer::{glib, Element, ElementFactory, Pipeline};

use glib::translate::ToGlibPtr;
// use glib_sys::{guint, GValue};
use gobject_sys::g_object_set_property;
use gobject_sys::GValue;

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

    fn apply_color_effect(invert: bool) -> Result<Vec<Element>, VideoError> {
        if invert {
            // https://gstreamer.freedesktop.org/documentation/coloreffects/coloreffects.html?gi-language=c
            // Color-effects-preset
            // The lookup table to use to convert input colors
            // Members
            // none (0) – Do nothing preset
            // heat (1) – Fake heat camera toning
            // sepia (2) – Sepia toning
            // xray (3) – Invert and slightly shade to blue
            // xpro (4) – Cross processing toning
            // yellowblue (5) – Yellow foreground Blue background color filter
            Ok(vec![
                ElementFactory::make("coloreffects")
                    .property_from_str("preset", "xray")
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
                ElementFactory::make("videoconvert")
                    .name("videoconvert1")
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ])
        } else {
            Ok(vec![])
        }
    }

    fn flip(flipflag: bool) -> Result<Vec<Element>, VideoError> {
        println!("GstreamerDecoder::flip: flipflag = {}", flipflag);
        if flipflag {
            // https://gstreamer.freedesktop.org/documentation/videofilter/videoflip.html?gi-language=c
            // method (deprecated, use video-direction instead)
            //
            // Default value : none (0)
            //
            // Members
            // none (0) – Identity (no rotation)
            // clockwise (1) – Rotate clockwise 90 degrees
            // rotate-180 (2) – Rotate 180 degrees
            // counterclockwise (3) – Rotate counter-clockwise 90 degrees
            // horizontal-flip (4) – Flip horizontally
            // vertical-flip (5) – Flip vertically
            // upper-left-diagonal (6) – Flip across upper left/lower right diagonal
            // upper-right-diagonal (7) – Flip across upper right/lower left diagonal
            // automatic (8) – Select flip method based on image-orientation tag

            Ok(vec![ElementFactory::make("videoflip")
                .name("videoflip0")
                .property_from_str("video-direction", "4")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?])
        } else {
            Ok(vec![])
        }
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

        let extra_steps = Self::hardcode_mp4_input()
            .and_then(|mut v| {
                let invert_steps = Self::apply_color_effect(opts.invert)?;
                v.extend(invert_steps);
                Ok(v)
            })
            .and_then(|mut v| {
                let change_res_steps = Self::change_res(opts.width_height)?;
                v.extend(change_res_steps);
                Ok(v)
            })
            .and_then(|mut v| {
                let flip_steps = Self::flip(opts.flip)?;
                println!("{} flip steps", flip_steps.len());
                v.extend(flip_steps);
                Ok(v)
            })?;

        decoder.steps.splice(1..1, extra_steps);

        decoder
            .pipeline
            .add_many(decoder.steps.iter())
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        for i in 0..decoder.steps.len() - 1 {
            println!(
                "Linking {} with {}",
                decoder.steps[i].name(),
                decoder.steps[i + 1].name()
            );

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
