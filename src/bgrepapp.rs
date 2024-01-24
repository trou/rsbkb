use crate::applet::Applet;
use anyhow::{bail, Context, Result};
use clap::{arg, Command};
use memmap2::Mmap;
use std::fs::{self, File};

use regex::bytes::{Regex, RegexBuilder};

/// Build the regex pattern with the given options.
/// By default, the `unicode` flag is set to false, and `dot_matches_new_line` set to true.
/// Code borrowed from gahag's bgrep (https://github.com/gahag/bgrep)
fn build_pattern<P: AsRef<str>>(pattern: &P) -> Result<Regex> {
    let mut builder = RegexBuilder::new(pattern.as_ref());

    builder.unicode(false);
    builder.dot_matches_new_line(true);
    builder
        .build()
        .with_context(|| "Could not build regular expression")
}

pub struct BgrepApplet {
    files: Option<Vec<String>>,
    pattern: Option<Regex>,
    verbose: bool,
}

impl Applet for BgrepApplet {
    fn command(&self) -> &'static str {
        "bgrep"
    }

    fn description(&self) -> &'static str {
        "binary grep"
    }

    fn returns_data(&self) -> bool {
        false
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-v --verbose  "verbose"))
            .arg(arg!(-x --hex  "pattern is hex"))
            .arg(arg!(<pattern>  "pattern to search"))
            .arg(arg!(<file>    "file to search").num_args(1..))
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            files: None,
            pattern: None,
            verbose: false,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let filenames = args
            .get_many::<String>("file")
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        let pattern_val = args.get_one::<String>("pattern").unwrap();

        /* Convert hex pattern to "\x00" format if needed */
        let mut s = String::new();
        let final_pat = if args.get_flag("hex") {
            if pattern_val.len() % 2 != 0 {
                bail!("hex pattern length is not even");
            }
            for i in 0..(pattern_val.len() / 2) {
                s += "\\x";
                s += &pattern_val[i * 2..i * 2 + 2];
            }
            s.as_str()
        } else {
            pattern_val
        };

        let pattern = build_pattern(&final_pat)?;

        Ok(Box::new(Self {
            files: Some(filenames),
            pattern: Some(pattern),
            verbose: args.get_flag("verbose"),
        }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let filenames = self.files.as_ref().unwrap();
        let many = filenames.len() > 1;
        for filename in filenames.iter() {
            if !fs::metadata(filename).is_ok_and(|f| f.is_file()) {
                if self.verbose {
                    eprintln!("Skipping non-file {}", filename);
                }
                continue;
            };

            let f = File::open(filename);
            match f {
                Ok(f) => {
                    /* Mmap is necessarily unsafe as data can change unexpectedly */
                    let data =
                        unsafe { Mmap::map(&f).with_context(|| "Could not mmap input file")? };

                    let regex = self.pattern.as_ref().unwrap();
                    let matches = regex.find_iter(&data);

                    /* Print offsets on stdout directly, to avoid buffering */
                    for m in matches {
                        if many {
                            println!("{}: 0x{:x}", filename, m.start());
                        } else {
                            println!("0x{:x}", m.start());
                        }
                    }
                }
                Err(e) => eprintln!("Could not open {}: {}", filename, e),
            }
        }

        /* Return empty Vec as we output directly on stdout */
        Ok(Vec::<u8>::new())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_cli() {
        let mut data: [u8; 10] = [0; 10];
        for i in (0..10).into_iter() {
            data[i] = i as u8;
        }

        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        tmpfile.write(&data).unwrap();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["bgrep", "-x", "020304", &tmpfile.path().to_str().unwrap()])
            .assert()
            .stdout("0x2\n")
            .success();
    }

    #[test]
    fn test_cli_multiple() {
        let mut tmpfile1 = tempfile::NamedTempFile::new().unwrap();
        tmpfile1.write(b"tmpfile1").unwrap();

        let mut tmpfile2 = tempfile::NamedTempFile::new().unwrap();
        tmpfile2.write(b"2tmpfile").unwrap();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&[
                "bgrep",
                "tmpfile",
                &tmpfile1.path().to_str().unwrap(),
                &tmpfile2.path().to_str().unwrap(),
            ])
            .assert()
            .stdout(predicates::str::contains(": 0x0\n"))
            .stdout(predicates::str::contains(": 0x1\n"))
            .success();
    }
}
