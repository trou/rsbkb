//#![feature(trace_macros)]
//trace_macros!(true);

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

pub use errors::*;

use std::io;
use std::io::{Read, Write};
extern crate base64;
extern crate clap;
use clap::Command;
extern crate crc;
extern crate hex;
extern crate percent_encoding;
use atty::Stream;

mod applet;
use applet::Applet;

mod hexapp;
use hexapp::HexApplet;
use hexapp::UnHexApplet;

mod urlapp;
use urlapp::UrlDecApplet;
use urlapp::UrlEncApplet;

mod b64app;
use b64app::B64DecApplet;
use b64app::B64EncApplet;

mod crcapp;
use crcapp::CRC16Applet;
use crcapp::CRC32Applet;
use crcapp::CRCApplet;

mod xorapp;
use xorapp::XorApplet;

mod sliceapp;
use sliceapp::SliceApplet;

mod timeapp;
use timeapp::TimeApplet;

mod patternapp;
use patternapp::BofPattGenApplet;
use patternapp::BofPattOffApplet;

mod entropyapp;
use entropyapp::EntropyApplet;

mod bgrepapp;
use bgrepapp::BgrepApplet;

mod findsoapp;
use findsoapp::FindSoApplet;

// Helper to "register" applets
macro_rules! applets {
    ($a:ident = $($x:ident),* )  =>
        {
            let $a : Vec<Box<dyn Applet>>= vec![$($x::new(),)*];

        };
}

fn main_with_errors() -> Result<()> {
    applets!(
        apps = HexApplet,
        UnHexApplet,
        UrlEncApplet,
        UrlDecApplet,
        CRC16Applet,
        CRC32Applet,
        CRCApplet,
        B64EncApplet,
        B64DecApplet,
        BofPattOffApplet,
        BofPattGenApplet,
        XorApplet,
        EntropyApplet,
        SliceApplet,
        BgrepApplet,
        FindSoApplet,
        TimeApplet
    );

    // Define a busybox-like multicall binary
    // Subcommands must be defined both as subcommands for "rsbkb" and
    // as main subcommands
    let mut app = clap::Command::new("rsbkb")
        .multicall(true)
        .version("1.0")
        .propagate_version(true)
        .subcommand(
            Command::new("rsbkb")
                .author("Raphaël Rigo <devel@syscall.eu>")
                .about("Rust BlackBag")
                .arg_required_else_help(true)
                .subcommands([Command::new("list").about("list applets")])
                .subcommand_value_name("APPLET")
                .subcommand_help_heading("APPLETS")
                .subcommands(apps.iter().map(|app| app.clap_command())),
        )
        .subcommands(apps.iter().map(|app| app.clap_command()));

    // Parse args
    let matches = app.get_matches_mut();

    /* Check if we're called as "rsbkb" */
    let subc = matches.subcommand_name();
    let real_matches = if subc == Some("rsbkb") {
        // get applet
        matches.subcommand().unwrap().1
    } else {
        &matches
    };

    // Get subcommand and args
    let (subcommand, sub_matches) = real_matches
        .subcommand()
        .chain_err(|| "Subcommand required")?;

    // list applets
    if subcommand == "list" {
        for app in apps.iter() {
            println!("{}", app.command());
        }
        return Ok(());
    }

    // Find corresponding app
    let selected_app = apps.iter().find(|a| a.command() == subcommand).unwrap();

    let mut inputval = vec![];

    if let Some(argname) = selected_app.arg_or_stdin() {
        /* Check if the given arg is present, else read from stdin */
        if !sub_matches.is_present(argname) {
            io::stdin()
                .read_to_end(&mut inputval)
                .expect("Reading stdin failed");
        } else {
            inputval = sub_matches.value_of(argname).unwrap().as_bytes().to_vec();
        }
    };

    let selected_app = selected_app.parse_args(sub_matches)?;

    let res = selected_app.process(inputval)?;

    if selected_app.returns_data() {
        let mut stdout = io::stdout();
        stdout.write_all(&res).expect("Write failed");

        /* Only add a newline when outputing to a terminal */
        if atty::is(Stream::Stdout) {
            println!();
        }
    }
    Ok(())
}

fn main() {
    let res = main_with_errors();
    if let Err(ref e) = res {
        use error_chain::ChainedError;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        let msg = e.display_chain().to_string();

        writeln!(stderr, "{}", msg.trim_end()).expect(errmsg);
        ::std::process::exit(1);
    }
}
