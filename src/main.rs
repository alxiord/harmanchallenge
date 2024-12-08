use std::path::PathBuf;

use clap::{builder::ValueParser, Arg, ArgAction, Command};

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

    // Why an Arc<Mutex> when we can't see any threads?
    // Because Rust is paranoid.
    // Somewhere in ::build, a closure is needed because the demuxer component can only be
    // linked to the next element at "runtime", i.e. when the pipeline starts playing.
    // So we register a callback for that, which implies a closure, which from Rust's point of
    // view can be executed on any other thread, and supersede the decoder instance's lifetime too.
    // Conceptually this scenario makes no sense but I can't defeat the compiler sooo, Arc<Mutex>
    // to enforce thread safety and avoid lifetime headaches
    let decoder_mutex = gst::GstreamerDecoder::new(infile.as_os_str().to_str().unwrap()).unwrap();
    GstreamerDecoder::build(decoder_mutex.clone(), DecoderOptions::default()).unwrap();
    // decoder.lock().unwrap().play().unwrap();
    let mut lock = decoder_mutex.lock();
    let decoder = lock.as_deref_mut().unwrap();
    decoder.run().unwrap();
}
