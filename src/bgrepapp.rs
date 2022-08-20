use crate::applet::Applet;
use clap::{App, arg, SubCommand};
use memmap2::Mmap;
use std::fs::File;
use std::process;

use regex::bytes::{Regex, RegexBuilder};

/// Build the regex pattern with the given options.
/// By default, the `unicode` flag is set to false, and `dot_matches_new_line` set to true.
/// Code borrowed from gahag's bgrep (https://github.com/gahag/bgrep)
fn build_pattern<P: AsRef<str>>(pattern: &P) -> Result<Regex, regex::Error> {
    let mut builder = RegexBuilder::new(pattern.as_ref());

    builder.unicode(false);
    builder.dot_matches_new_line(true);
    builder.build()
}

pub struct BgrepApplet {
    file: Option<String>,
    pattern: Option<Regex>,
}

impl Applet for BgrepApplet {
    fn command(&self) -> &'static str {
        "bgrep"
    }
    fn description(&self) -> &'static str {
        "binary grep"
    }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command())
            .about(self.description())
            .arg(arg!(-x --hex  "pattern is hex"))
            .arg(arg!(<pattern>  "pattern to search"))
            .arg(arg!(<file>    "file to search"))
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            file: None,
            pattern: None,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let filename = args.value_of("file").unwrap();
        let pattern_val = args.value_of("pattern").unwrap();

        /* Convert hex pattern to "\x00" format if needed */
        let mut s = String::new();
        let final_pat = if args.is_present("hex") {
            if pattern_val.len() % 2 != 0 {
                eprintln!("Error: hex pattern length is not even");
                process::exit(1);
            }
            for i in 0..(pattern_val.len() / 2) {
                s += "\\x";
                s += &pattern_val[i * 2..i * 2 + 2];
            }
            s.as_str()
        } else {
            pattern_val
        };

        let pattern = build_pattern(&final_pat).expect("Invalid regex");

        Box::new(Self {
            file: Some(filename.to_string()),
            pattern: Some(pattern),
        })
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let filename = self.file.as_ref().unwrap();
        let f = File::open(filename).expect("Cannot open file");

        /* Mmap is necessarily unsafe as data can change unexpectedly */
        let data = unsafe { Mmap::map(&f).expect("Could not mmap input file") };

        let regex = self.pattern.as_ref().unwrap();
        let matches = regex.find_iter(&data);
        let mut do_cr = false;

        /* Print offsets on stdout directly, to avoid buffering */
        for m in matches {
            /* last line should not have a \n as we add one in main */
            if do_cr {
                println!();
            } else {
                do_cr = true
            };
            print!("0x{:x}", m.start());
        }

        /* Return empty Vec as we output directly on stdout */
        Vec::<u8>::new()
    }
}
