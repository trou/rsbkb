//#![feature(trace_macros)]
//trace_macros!(true); 
use std::env;
use std::io;
use std::io::{Read, Write};
//use std::fs::{OpenOptions};
//use std::fs;
extern crate hex;
extern crate base64;
extern crate percent_encoding;
extern crate clap;
extern crate crc;
use atty::Stream;

// Allow the use of as_bytes() on OsStr
use std::os::unix::ffi::OsStrExt;

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
                    B64EncApplet, B64DecApplet);
    let app_names : Vec<_> = apps.iter().map(|app| app.command()).collect();

    /* Get arg0 */
    let mut args: Vec<_>= env::args_os().collect();
    let arg0 = env::current_exe().expect("Can't get executable name :");
    let arg0 = arg0.file_name().expect("No filename for executable filename ?");

    let mut app = clap::App::new("rsbkb")
        .version("0.3.0")
        .author("Raphael Rigo <devel@syscall.eu>")
        .about("Rust BlackBag")
        .subcommands(apps.iter().map(|app| app.subcommand()));

    // Check if arg0 is a supported subcommand and use it
    let arg0_str = &arg0.to_str().unwrap_or("");
    if app_names.contains(arg0_str) {
        args.insert(1, arg0.to_os_string());
    }

    // Parse args
    let matches = app.clone().get_matches_from(args);

    // Get subcommand and args
    let (subcommand, sub_matches) = match matches.subcommand() {
        (s, Some(sm)) => (s, sm),
        _ => { &app.print_help(); println!(""); return;}
    };

    // Find corresponding app
    let selected_app = apps.iter().find(|a| a.command() == subcommand).unwrap();

    let mut inputval = vec![];

    if let Some(argname) = selected_app.arg_or_stdin() {
        /* Check if the given arg is present, else read from stdin */
        if !sub_matches.is_present(argname) {
            io::stdin().read_to_end(&mut inputval).expect("Reading stdin failed");
        } else {
            inputval = sub_matches.value_of_os(argname).unwrap().as_bytes().to_vec();
        }
    };

    let selected_app = selected_app.parse_args(sub_matches);
    let res = selected_app.process(inputval);

    let mut stdout = io::stdout();
    stdout.write(&res).expect("Write failed");

    /* Only add a newline when outputing to a terminal */
    if atty::is(Stream::Stdout) {
        println!("");
    }
}

