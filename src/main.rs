//#![feature(trace_macros)]
//trace_macros!(true);

use anyhow::{anyhow, Result};

use std::io::{self, IsTerminal};
use std::io::{Read, Write};
use std::path::Path;
extern crate base64;
extern crate clap;
use clap::Command;
extern crate crc;
extern crate hex;
extern crate percent_encoding;

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

mod flateapp;
use flateapp::DeflateApplet;
use flateapp::InflateApplet;

mod baseapp;
use baseapp::BaseIntApplet;

// Helper to "register" applets
macro_rules! applets {
    ($a:ident = $($x:ident),* )  =>
        {
            let $a : Vec<Box<dyn Applet>>= vec![$($x::new(),)*];

        };
}

fn main() -> Result<()> {
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
        TimeApplet,
        DeflateApplet,
        InflateApplet,
        BaseIntApplet
    );

    // Define a busybox-like multicall binary
    // Subcommands must be defined both as subcommands for "rsbkb" and
    // as main subcommands
    let mut app = clap::Command::new("rsbkb")
        .multicall(true)
        .version(env!("CARGO_PKG_VERSION"))
        .propagate_version(true)
        .subcommand(
            Command::new("rsbkb")
                .help_template(
                    "\
{before-help}{name} {version} ({about}) - by {author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
",
                )
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
        .ok_or_else(|| anyhow!("Subcommand required"))?;

    // list applets
    if subcommand == "list" {
        for app in apps.iter() {
            println!("{}", app.command());
        }
        return Ok(());
    }

    // Find corresponding app
    let selected_app = apps.iter().find(|a| a.command() == subcommand).unwrap();

    // Parse applet args and get actual applet with options
    let selected_app = selected_app.parse_args(sub_matches)?;

    let mut inputval = vec![];

    if let Some(argname) = selected_app.arg_or_stdin() {
        /* Check if the given arg is present, else read from stdin */
        if !sub_matches.contains_id(argname) {
            io::stdin()
                .read_to_end(&mut inputval)
                .expect("Reading stdin failed");
        } else {
            /* Check if the given argument could be a filename, which is probably not
             * what the user wants */
            let argname_val: &String = sub_matches.get_one::<String>(argname).unwrap();
            if Path::new(argname_val).exists() {
                eprintln!(
                    "'{}' is a file, maybe you want to pass it to stdin instead?",
                    argname_val
                );
            }
            inputval = argname_val.as_bytes().to_vec();
        }
    };

    let res = selected_app.process(inputval)?;

    if selected_app.returns_data() {
        let mut stdout = io::stdout();
        let write_res = stdout.write_all(&res);

        // Ignore broken pipe
        match write_res {
            Err(err) if err.kind() != std::io::ErrorKind::BrokenPipe => {
                return Err(err.into());
            }
            Err(_) => {
                return Ok(());
            }
            Ok(_) => (),
        };

        /* Only add a newline when outputing to a terminal */
        if std::io::stdout().is_terminal() {
            println!();
        }
    }
    Ok(())
}
