# Harman tech challenge

## Requirements

- Input: video stream captured from camera (the one from your laptop should be enough; use its default video format)
- Output: processed video stream displayed on a screen (using a screen as sink would allow us to preserve the performance; basically, we aim for sort of a real time transformation which preserves the frame rate of the input)
- Processing: The application which is fed with the input above shall perform the following actions:
  - Force the format at the output to H264
  - Scale the video
  - Invert colors
  - Flip it
- Deliverables: README (compiling instructions), build system (makefile, autotools, or whatever you prefer), sources (in any collab tool you prefer, your GitHub would be great).
- Misc: A compiled programming language shall be used.

## Brain dump

- use `gstreamer`
- dockerize
- add `ffmpeg` if there's time

## gstreamer

### Extra dependencies

```bash
apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio
```

Pipeline to just view the video:

```bash
gst-launch-1.0 -v filesrc location=input/hello.mp4 ! qtdemux ! h264parse ! avdec_h264 ! videoconvert ! xvimagesink

# UPDATE SIMPLIFIED
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

- to apply coloreffect: add `coloreffects preset=3 ! videoconvert` after first `videoconvert`
- to flip: add `videoflip method=horizontal-flip !` after last `videoconvert`
- to resize: add `videoscale ! video/x-raw,width=600,height=400` after last `videoconvert`

BUT - to add h264 encoding but **not** save to file: don't know how to do this

### trying out videoinvert

This plugin should exist in the gst-plugins-good package, but on my setup despite retrying loads of times
it just doesn't show up. Let's build it.

````bash
git clone https://gitlab.freedesktop.org/gstreamer/gstreamer.git
cd gstreamer/subprojects/gst-plugins-good
git checkout 1.20.3

sudo apt-get install build-essential meson ninja-build libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev

meson builddir
ninja -C builddir

WHATEVER GIVE UP
```

### cmdline invocation

Gotta fix these positional args...

```bash
cargo run --  -i input/hello.mp4 -w 600 -h 400 h264 true
```

## Build with Docker

```bash
docker run -it -v $PWD:/home/alexandra harmanchallenge:v3 bash
````

and inside the container

```bash
cargo build
cargo run -- -i input/hello.mp4
```
