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

- use `ffmpeg` FFI initially, let `ffmpeg` do the heavy lifting and safely wrap it wherever possible in nice Rust code
- dockerize
- check in with team and, if necessary, do v2 rewriting the `ffmpeg` black magic in Rust

## Dependencies

- gstreamer:

  ```bash
  apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev
  ```

## Build with Docker

```bash
docker run -it -v $PWD:/home/alexandra harmanchallenge:v3 bash
```

and inside the container

```bash
cargo build
cargo run -- -i input/hello.mp4
```
