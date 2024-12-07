use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, Command};

fn main() {
    println!("Hello, world!");

    let cmd = Command::new("harmanchallenge")
        .arg(
            Arg::new("input")
                .short('i')
                .required(true)
                .value_parser(NonEmptyStringValueParser::new())
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("format")
                .default_value("h264")
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
    let infile = matches.get_one::<String>("input").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let width = matches.get_one::<u32>("width").unwrap_or(&0);
    let height = matches.get_one::<u32>("height").unwrap_or(&0);
    let invert = matches.contains_id("invert");
    let flip = matches.contains_id("flip");

    println!(
        "Input file: {}\nFormat: {}\nW: {} H: {}\nInvert {}\nFlip {}",
        infile, format, width, height, invert, flip
    );
}
