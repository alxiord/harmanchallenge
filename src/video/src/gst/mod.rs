use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, PadExt};
use gstreamer::{glib, Element, ElementFactory, Pipeline};

use util::{DecoderOptions, VideoFormat};

use crate::VideoInput;

use super::Error as VideoError;

#[derive(Debug)]
/// Gstreamer errors
pub enum Error {
    /// glib error
    Glib(glib::Error),
    /// glib error
    GlibBool(glib::BoolError),
    /// Error occurred while changing the gstreamer pipeline state
    PipelineStateChange(gstreamer::StateChangeError),
    /// gstreamer pipeline doesn't have a message bus
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

/// Struct that implements the [`Decoder`](crate::Decoder) trait using gstreamer as a backend
pub struct GstreamerDecoder {
    srcsteps: Vec<Element>,
    sinksteps: Vec<Element>,
    pipeline: Pipeline,
}

impl GstreamerDecoder {
    /// Create the first steps of the pipeline, hardcoded for parsing mp4 files:
    /// 1. [filesrc](https://gstreamer.freedesktop.org/documentation/coreelements/filesrc.html?gi-language=c)
    /// 1. [demuxer](https://gstreamer.freedesktop.org/documentation/isomp4/qtdemux.html?gi-language=c) that splits the mp4 file into video and audio streams
    /// 1. [`h264`` decoder](https://gstreamer.freedesktop.org/documentation/libav/avdec_h264.html?gi-language=c#avdec_h264-page) for the demux'ed video stream
    /// 1. [video converter](https:/)/gstreamer.freedesktop.org/documentation/videoconvertscale/videoconvert.html?gi-language=c#videoconvert-page) to automatically convert the video stream into a format
    ///    compatible with whatever comes next in the pipeline
    fn filesource(infname: String) -> Result<Vec<Element>, VideoError> {
        Ok(vec![
            ElementFactory::make("filesrc")
                .property_from_str("location", infname.as_str())
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
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
        ])
    }

    fn webcamsource() -> Result<Vec<Element>, VideoError> {
        // v4l2src ! videoconvert !
        Ok(vec![
            ElementFactory::make("v4l2src")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ElementFactory::make("videoconvert")
                .name("videoconvert0")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
        ])
    }

    fn source(input: VideoInput) -> Result<Vec<Element>, VideoError> {
        match input {
            VideoInput::File(fname) => Self::filesource(fname),
            VideoInput::Webcam => Self::webcamsource(),
        }
    }

    /// Create steps for changing the width and height of the video:
    /// 1. [`videoscale`](https://gstreamer.freedesktop.org/documentation/videoconvertscale/videoscale.html?gi-language=c#videoscale-page) for resizing the video frames
    /// 1. [`capsfilter`](https://gstreamer.freedesktop.org/documentation/coreelements/capsfilter.html?gi-language=c#capsfilter-page) for specifying the desired width and height
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

    /// Create steps for changing the color of the video (note that the filter used here, "xray",
    /// adds a blue hue after inverting - `gstreamer` doesn't have a "just invert" filter):
    /// 1. [`coloreffects`](https://gstreamer.freedesktop.org/documentation/coloreffects/coloreffects.html?gi-language=c) for applying the `xray` effect
    /// 1. [video converter](https:/)/gstreamer.freedesktop.org/documentation/videoconvertscale/videoconvert.html?gi-language=c#videoconvert-page) to automatically convert the video stream into a format
    ///    compatible with whatever comes next in the pipeline
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

    /// Create steps for flipping the video horizontally:
    /// 1. [`videoflip`](https://gstreamer.freedesktop.org/documentation/videofilter/videoflip.html?gi-language=c)
    fn flip(flipflag: bool) -> Result<Vec<Element>, VideoError> {
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

    fn encode(format: VideoFormat) -> Result<Vec<Element>, VideoError> {
        //   x264enc tune=zerolatency ! queue ! avdec_h264 ! videoconvert !
        match format {
            VideoFormat::H264 => Ok(vec![
                ElementFactory::make("x264enc")
                    .name("x264enc0")
                    .property_from_str("tune", "zerolatency")
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
                ElementFactory::make("queue")
                    .name("queue0")
                    .build()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ]),
        }
    }

    fn screenout() -> Result<Vec<Element>, VideoError> {
        Ok(vec![
            ElementFactory::make("avdec_h264")
                .name("avdec_h2641")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ElementFactory::make("videoconvert")
                .name("videoconvert2") // todo: keep a map
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
            ElementFactory::make("xvimagesink")
                .name("xvimagesink0")
                .build()
                .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?,
        ])
    }

    /// Callback for linking the demuxer (dynamically) when the pipeline starts playing.
    /// The [`qtdemux`](https://gstreamer.freedesktop.org/documentation/qtdemux/qtdemux.html?gi-language=c) element can't be
    /// linked to the next element during pipeline creation, hence the need to register a callback
    /// and handle it dynamically at "run"time.
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
    /// Create the file source and sink elements that delimit the pipeline
    fn new(input: VideoInput) -> Result<Arc<Mutex<Self>>, VideoError> {
        gstreamer::init().map_err(|e| VideoError::Gstreamer(Error::Glib(e)))?;

        Ok(Arc::new(Mutex::new(GstreamerDecoder {
            srcsteps: Self::source(input)?,
            sinksteps: Self::screenout()?,
            pipeline: Pipeline::with_name("hc-pipeline"),
        })))
    }

    /// Build the gstreamer pipeline.
    /// When all the supported filters are added, the pipeline looks like this:
    ///
    /// ```
    /// {source} - {coloreffects} - {videoconvert} - {videoscale} - {capsfilter} - {videoflip} - {encode} {xvimgsink}
    /// ```
    fn build(self_rc: Arc<Mutex<Self>>, opts: DecoderOptions) -> Result<(), VideoError> {
        let mut lock = self_rc.lock();
        let decoder = lock.as_deref_mut().map_err(|_| VideoError::PoisonedLock)?;

        let filter_steps = Self::apply_color_effect(opts.invert)
            .and_then(|mut v| {
                let resize_steps = Self::change_res(opts.width_height)?;
                v.extend(resize_steps);
                Ok(v)
            })
            .and_then(|mut v| {
                let flip_steps = Self::flip(opts.flip)?;
                v.extend(flip_steps);
                Ok(v)
            })
            .and_then(|mut v| {
                let encode_steps = Self::encode(opts.format)?;
                v.extend(encode_steps);
                Ok(v)
            })?;

        let mut all_steps: Vec<Element> = decoder.srcsteps.clone();
        all_steps.extend(filter_steps);
        all_steps.extend(decoder.sinksteps.clone());

        decoder
            .pipeline
            .add_many(all_steps.iter())
            .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;

        for i in 0..all_steps.len() - 1 {
            println!(
                "Linking {} with {}",
                all_steps[i].name(),
                all_steps[i + 1].name()
            );

            if all_steps[i].name() == "demux" {
                // Special handling for demux!!
                // Why?
                // Because as the name suggests it *demultiplexes* src into multiple streams,
                // and the next element can't know what to link to unless explicitly shown.
                //
                // inspo:
                // https://stackoverflow.com/a/65591800
                // https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/0b1be1178918166a2e519d82f2935d68034ad046/examples/src/bin/transmux.rs
                let next_elem = all_steps[i + 1].clone();

                all_steps[i].connect_pad_added(move |_demux, src_pad| {
                    // let self_clone = Rc::clone(&self_clone);
                    let next_elem = next_elem.clone();
                    GstreamerDecoder::handle_demux_pad_added(&src_pad, &next_elem);
                });

                all_steps[i]
                    .sync_state_with_parent()
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;
            } else {
                all_steps[i]
                    .link(&all_steps[i + 1])
                    .map_err(|e| VideoError::Gstreamer(Error::GlibBool(e)))?;
            }
        }

        Ok(())
    }

    /// Play the pipeline (run the video through the filters and play it on the screen)
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
