#![crate_name = "util"]
#![deny(missing_docs)]

//! # Utilities

use clap::{command, Parser};

use std::error;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

/// Argument validators
pub mod validator;

#[derive(Copy, Clone, Debug)]
/// Supported output video formats
pub enum VideoFormat {
    /// Represents the h264 format
    H264,
}

impl Display for VideoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoFormat::H264 => write!(f, "h264"),
        }
    }
}

#[derive(Debug)]
/// Errors that can occur while parsing the cmdline arguments
pub enum Error {
    /// I/O error
    Io(io::Error),
    /// Unsupported format
    Format(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Format(e) => write!(f, "Invalid format: {}", e),
        }
    }
}

impl error::Error for Error {}

#[derive(Parser, Debug)]
#[command(name = "harman-challenge")]
/// Command line arguments definition
pub struct Cli {
    #[arg(long, value_parser = validator::parse_fname)]
    /// Input video file
    pub input: Option<PathBuf>,
    #[arg(long, value_parser = validator::parse_format)]
    /// Output video format
    format: Option<VideoFormat>,
    #[arg(long)]
    /// Output video width
    width: Option<i32>,
    #[arg(long)]
    /// Output video height
    height: Option<i32>,
    #[arg(long)]
    /// Flag that specifies whether the output file should be inverted
    invert: bool,
    #[arg(long)]
    /// Flag that specifies whether the output file should be flipped horizontally
    flip: bool,
}

#[derive(Clone, Copy, Debug)]
/// Video manipulator options
pub struct DecoderOptions {
    /// Output resolution (width x height)
    pub width_height: Option<(i32, i32)>,
    /// Flag that specifies whether the output file should be inverted
    pub invert: bool,
    /// Flag that specifies whether the output file should be flipped horizontally
    pub flip: bool,
}

impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            width_height: None,
            invert: false,
            flip: false,
        }
    }
}

impl From<&Cli> for DecoderOptions {
    fn from(cli: &Cli) -> Self {
        let mut opts = DecoderOptions::default();
        if let Some(w) = cli.width {
            if let Some(h) = cli.height {
                opts.width_height = Some((w, h));
            }
        }
        opts.invert = cli.invert;
        opts.flip = cli.flip;
        opts
    }
}
