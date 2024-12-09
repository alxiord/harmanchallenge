# Harman tech challenge

## Introduction

This project consists in a program with the following capabilities:

- open a video file
  - **Note:** for simplicity, the only supported format is `mp4` (as saved by my laptop's webcam software).
- force the format to `h264`
- scale the video to a given width and height
- invert the colors
- flip the video horizontally

## Deliverables

- Rust project implementing the requirements above
- Docker image with all the dependencies (Rust toolchain, `gstreamer` libraries) preinstalled
- Python wrapper script
- Documentation (this + autogeneratable rustdoc HTML pages)

## Rationale

- Why Rust: it's the compiled language I'm most comfortable with
- Why Docker: so you don't have to install the Rust toolchain and the `gstreamer` libraries
- Why Python: because the `cargo` commands can be tricky when unfamiliar with Rust, so a more friendly wrapper was needed
- Why `gstreamer`: after briefly considering `gstreamer` and `ffmpeg`, I chose the latter because the pipeline
  architecture is easy to understand and it made it easy for me, knowing nothing about video manipulation, to
  conceptualize how filters can be chained together and what the information flow looks like.

## Architecture Overview

### `video` crate

The `video` crate is the core of the project. It contains the Rust code that implements the video processing logic.
It defines a trait (Rust "interface") which outlines the capabilities of a video processor:

```rust
/// Trait that defines the common interface for supported video manipulator structs
pub trait Decoder {
    /// Create a new instance
    fn new(infname: &str) -> Result<Arc<Mutex<Self>>, Error>;
    /// Add decoders, encoders and filters
    fn build(self_rc: Arc<Mutex<Self>>, opts: DecoderOptions) -> Result<(), Error>;
    /// Parse the input file and output the result to the screen
    fn run(&mut self) -> Result<(), Error>;
}
```

The `DecoderOptions` struct encapsulates the command line parameters in order to tailor how the video processor
will look like.

```rust
pub struct DecoderOptions {
    /// Output resolution (width x height)
    pub width_height: Option<(i32, i32)>,
    /// Flag that specifies whether the output file should be inverted
    pub invert: bool,
    /// Flag that specifies whether the output file should be flipped horizontally
    pub flip: bool,
}
```

#### `video.gst` module

This module contains the Rust code that implements the `Decoder` trait for the `gstreamer` library.
Each of the `Decoder` trait's functions is implemented as one or more `gstreamer` plugins.
In `build()`, they are linked into a [`gstreamer` pipeline](https://gstreamer.freedesktop.org/documentation/application-development/introduction/basics.html?gi-language=c#bins-and-pipelines).
When played, the pipeline opens and parses the input video, applies effects and outputs it to the screen.

##### `new()`

This function creates a new [`gstreamer` pipeline](https://gstreamer.freedesktop.org/documentation/application-development/introduction/basics.html?gi-language=c#bins-and-pipelines) and it's "head" and "tail"
[elements](https://gstreamer.freedesktop.org/documentation/application-development/introduction/basics.html?gi-language=c#elements):

- [`filesrc`](https://gstreamer.freedesktop.org/documentation/coreelements/filesrc.html?gi-language=c#filesrc-page):
  this element reads the input file and passes it on to the filter elements that will follow
- [`xvimagesink`](https://gstreamer.freedesktop.org/documentation/xvimagesink/index.html?gi-language=c#xvimagesink-page):
  this will be the last element of the pipeline, and it will display the output video on the screen

##### pipeline elements

The following elements are added to the pipeline:

- `mp4` input handling:
  - [`qtdemux`](https://gstreamer.freedesktop.org/documentation/qtdemux/index.html?gi-language=c#qtdemux-page):
    this demultiplexes the input file into a video and an audio stream. The video stream proceeds to filtering
  - [`avdec_h264`](https://gstreamer.freedesktop.org/documentation/libav/avdec_h264.html?gi-language=c#avdec_h264-page):
    this decodes the video stream assuming `h264` encoding
  - [`videoconvert`](https://gstreamer.freedesktop.org/documentation/videoconvert/index.html?gi-language=c#videoconvert-page):
    this autoconverts the video stream to something compatible with the next element in the pipeline
- color inversion - **optional**:
  - [`coloreffects`](https://gstreamer.freedesktop.org/documentation/coloreffects/index.html?gi-language=c#coloreffects-page):
    this applies a color filter using a predefined preset on the video stream. To invert the colors, the `xray` preset is used, which is not quite exactly what requested but close enough. More details in the [#Appendix](#Appendix).
  - [`videoconvert`](https://gstreamer.freedesktop.org/documentation/videoconvert/index.html?gi-language=c#videoconvert-page):
    this autoconverts the video stream to something compatible with the next element in the pipeline
- resolution change - **optional**:
  - [`videoscale`](https://gstreamer.freedesktop.org/documentation/videoconvertscale/videoscale.html?gi-language=c#videoscale-page):
    this resizes the video frames to the spepecified width and height
- horizontal flip - **optional**:
  - [`videoflip`](https://gstreamer.freedesktop.org/documentation/videofilter/videoflip.html?gi-language=c):
    this plugin flips the video stream with a predefined preset for direction. According to the official documentation,
    preset 4 is for horizontal flipping.
- sink:
  - [`xvimagesink`](https://gstreamer.freedesktop.org/documentation/xvimagesink/index.html?gi-language=c#xvimagesink-page):
    this renders the resulting frames on the screen using the xvideo extension.

##### `build()`

This function adds all the aforementioned elements to the pipeline, instantiating each filter only
if so specified in the command line args. The elements are then linked together (with special care for
the demuxer, whose implementation requires dynamic callback-based linking)

##### `run()`

This function sets the pipeline to `playing` state and runs it, rendering the filtered video.
The program exits cleanly when the video ends, terminating the pipeline.

## Appendix

### Extra dependencies

`gstreamer` brings in several dependencies which encapsulate the actual implementation of the
video processing magic.
An overly bloated list is:

```bash
apt-get install gstreamer1.0-plugins-base gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio  \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev                \
  gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly                        \
  gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl
```

Note that I'm quite sure not _all_ oh these are needed, but I haven't checked.
Moreover, the Docker image already has them preinstalled.

### Useful `gstreamer` debug commands

Pipeline to just view the video:

```bash
gst-launch-1.0 -v filesrc location=input/hello.mp4 ! qtdemux ! h264parse ! avdec_h264 ! videoconvert ! xvimagesink
```

Or, letting `gstreamer` figure it out for themselves:

```bash
gst-launch-1.0 -v filesrc location=input/hello.mp4 ! decodebin ! xvimagesink
```

To flip:

```bash
// ... videoflip method=horizontal-flip !  xvimagesink
```

To invert (xray actually):

```bash
gst-launch-1.0 filesrc location=input/hello.mp4 ! qtdemux name=demux demux.video_0 ! avdec_h264 ! videoconvert ! coloreffects preset=3 ! videoconvert ! videoflip method=horizontal-flip ! xvimagesink
```

To resize:

```bash
gst-launch-1.0 filesrc location=input/hello.mp4 ! qtdemux name=demux   demux.video_0 ! avdec_h264 ! videoconvert ! coloreffects preset=3 ! videoconvert ! videoscale ! video/x-raw,width=600,height=400 ! videoflip method=horizontal-flip ! xvimagesink
```

BUT - to add h264 encoding but **not** save to file: don't know how to do this

### trying out `videoinvert`

One of the main drawbacks with the current solution is that the `--invert` flag doesn't _actually_ invert
the colors, instead it applies the `xray` preset of `gstreamer`'s `coloreffects` plugin.
Turns out there is no preset to just invert the colors, without adding the blueish tint included in `xray`.
There is, however, a `videoinvert` plugin that should do just that, only I haven't been able to get it to work.

This plugin should exist in the `gst-plugins-good package`, but on my setup despite retrying loads of times
it just doesn't show up. I also tried to build it:

```bash
git clone https://gitlab.freedesktop.org/gstreamer/gstreamer.git
cd gstreamer/subprojects/gst-plugins-good
git checkout 1.20.3

sudo apt-get install build-essential meson ninja-build libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev

meson builddir
ninja -C builddir
...
```

...but ended up giving up because of some incompatible libraries.

### cmdline invocation

```bash
cargo run -- --input=input/hello.mp4 --width=600 --height=400 --format=h264 --flip --invert
```

## Build with Docker

```bash
docker run -it -v $PWD:/home/alexandra harmanchallenge:v3 bash
```

inside the container

```bash
cargo build [--release]
```
