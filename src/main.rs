//#![feature(trace_macros)]
//trace_macros!(true);
use std::env;
use std::io;
use std::path::Path;
use std::io::{Read, Write};
extern crate hex;
extern crate base64;
extern crate percent_encoding;
extern crate clap;
extern crate crc;
use atty::Stream;

mod applet;
use applet::Applet;

mod hexapp;
use hexapp::HexApplet;
use hexapp::UnHexApplet;

mod urlapp;
use urlapp::UrlEncApplet;
use urlapp::UrlDecApplet;

mod b64app;
use b64app::B64EncApplet;
use b64app::B64DecApplet;

mod crcapp;
use crcapp::CRC16Applet;
use crcapp::CRC32Applet;

mod xorapp;
use xorapp::XorApplet;

mod sliceapp;
use sliceapp::SliceApplet;

mod timeapp;
use timeapp::TimeApplet;

mod patternapp;
use patternapp::BofPattOffApplet;
use patternapp::BofPattGenApplet;

mod entropyapp;
use entropyapp::EntropyApplet;

// Helper to "register" applets
macro_rules! applets {
    ($a:ident = $($x:ident),* )  =>
        {
            let $a : Vec<Box<dyn Applet>>= vec![$($x::new(),)*];

        };
}

fn main() {
    applets!(apps = HexApplet, UnHexApplet,
                    UrlEncApplet, UrlDecApplet,
                    CRC16Applet, CRC32Applet,
                    B64EncApplet, B64DecApplet,
                    BofPattOffApplet, BofPattGenApplet,
                    XorApplet, EntropyApplet,
                    SliceApplet,
                    TimeApplet);
    let app_names : Vec<_> = apps.iter().map(|app| app.command()).collect();

    /* Get arg0 */
    let mut args: Vec<_>= env::args().collect();

    let arg0 = Path::new(&args[0]).file_name();
    let arg0 = match arg0 {
        Some(a) => a.to_str().unwrap().to_string(),
        None => panic!("No arg0"),
    };

    let mut app = clap::App::new("rsbkb")
        .version("0.6.1")
        .author("Raphael Rigo <devel@syscall.eu>")
        .about("Rust BlackBag")
        .subcommands(apps.iter().map(|app| app.subcommand()));

    // Check if arg0 is a supported subcommand and use it
    if app_names.contains(&arg0.as_str()) {
        args.insert(1, arg0);
    }

    // Parse args
    let matches = app.clone().get_matches_from(args);

    // Get subcommand and args
    let (subcommand, sub_matches) = match matches.subcommand() {
        (s, Some(sm)) => (s, sm),
        _ => { app.print_help().expect("Help failed ;)"); println!(); return;}
    };

    // Find corresponding app
    let selected_app = apps.iter().find(|a| a.command() == subcommand).unwrap();

    let mut inputval = vec![];

    if let Some(argname) = selected_app.arg_or_stdin() {
        /* Check if the given arg is present, else read from stdin */
        if !sub_matches.is_present(argname) {
            io::stdin().read_to_end(&mut inputval).expect("Reading stdin failed");
        } else {
            inputval = sub_matches.value_of(argname).unwrap().as_bytes().to_vec();
        }
    };

    let selected_app = selected_app.parse_args(sub_matches);
    let res = selected_app.process(inputval);

    let mut stdout = io::stdout();
    stdout.write_all(&res).expect("Write failed");

    /* Only add a newline when outputing to a terminal */
    if atty::is(Stream::Stdout) {
        println!();
    }
}

