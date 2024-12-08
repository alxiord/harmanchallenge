use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use clap::{builder::ValueParser, Arg, ArgAction, Command};
use lazy_static::lazy_static;

use util::{validator, VideoFormat};
use video::{
    gst::{self, GstreamerDecoder},
    Decoder, DecoderOptions,
};

fn main() {
    println!("Hello, world!");

    let cmd = Command::new("harmanchallenge")
        .arg(
            Arg::new("input")
                .short('i')
                .required(true)
                .value_parser(ValueParser::new(validator::parse_fname))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("format")
                .default_value("h264")
                .value_parser(ValueParser::new(validator::parse_format))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("height")
                .short('h')
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set),
        )
        .arg(Arg::new("invert"))
        .arg(Arg::new("flip"))
        .disable_help_flag(true);

    let matches = cmd.get_matches();

    // Safe to unwrap here - if some of the required args are missing, it doesn't make sense for the program to run
    let infile = matches.get_one::<PathBuf>("input").unwrap();
    let format = matches.get_one::<VideoFormat>("format").unwrap();
    let width = matches.get_one::<u32>("width");
    let height = matches.get_one::<u32>("height");
    let invert = matches.contains_id("invert");
    let flip = matches.contains_id("flip");

    println!(
        "Input file: {:?}\nFormat: {}\nW: {:?} H: {:?}\nInvert {}\nFlip {}",
        infile.as_os_str(),
        format,
        width,
        height,
        invert,
        flip
    );

    let decoder = gst::GstreamerDecoder::new().unwrap();
    // decoder.build(DecoderOptions::default()).unwrap();
    // decoder
    //     .lock()
    //     .unwrap()
    //     .build(DecoderOptions {
    //         width: width.copied(),
    //         height: height.copied(),
    //         invert,
    //         flip,
    //     })
    //     .unwrap();
    GstreamerDecoder::build(decoder.clone(), DecoderOptions::default()).unwrap();
}
