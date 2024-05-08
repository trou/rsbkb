use crate::applet::Applet;
use anyhow::{bail, Context, Result};
use clap::{arg, Command};
use memmap2::Mmap;
use std::{collections::BTreeSet, fs::{self, read_dir, File}, path::PathBuf};

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
    recursive: bool
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
            .arg(arg!(-r --recursive "search in subfolders"))
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
            recursive: false
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
            recursive: args.get_flag("recursive")
        }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let input_paths = self.files.as_ref().unwrap();
        let many = input_paths.len() > 1 || self.recursive;
        // Make sure we keep the search order based on what is given first as the input
        for input_path in input_paths.iter() {
            // A BTreeSet ensure we get a consistant order
            let mut paths_to_explore = BTreeSet::new();
            paths_to_explore.insert(PathBuf::from(input_path));

            while let Some(path) = paths_to_explore.pop_first() {
                let path_metadata = match fs::metadata(&path) {
                    Ok(x) => x,
                    Err(err) => {
                        eprintln!("Skiping {} with non-obtainable metadata ({})", path.to_string_lossy(), err);
                        continue;
                    }
                };

                if path_metadata.is_file() {
                    let f = File::open(&path);
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
                                    println!("{}: 0x{:x}", path.to_string_lossy(), m.start());
                                } else {
                                    println!("0x{:x}", m.start());
                                }
                            }
                        }
                        Err(e) => eprintln!("Could not open {}: {}", path.to_string_lossy(), e),
                    }
                } else if path_metadata.is_dir() {
                    if !self.recursive {
                        if self.verbose {
                            eprintln!("Skipping directory {}", path.to_string_lossy())
                        }
                        continue;
                    }

                    let dir_read = match read_dir(&path) {
                        Ok(x) => x,
                        Err(err) => {
                            eprintln!("Skipping directory {}, failed to list childs ({})", path.to_string_lossy(), err);
                            continue;
                        }
                    };
                    for sub_path_unchecked in dir_read {
                        let sub_path = match sub_path_unchecked {
                            Ok(x) => x,
                            Err(err) => {
                                eprintln!("Skipping a sub-path of directory {}, failed to list a child ({})", path.to_string_lossy(), err);
                                continue;
                            }
                        };
                        paths_to_explore.insert(sub_path.path());
                    }
                } else {
                    if self.verbose {
                        eprintln!("Skipping non-file {}", path.to_string_lossy());
                    }
                    continue;
                }
            }
        }

        /* Return empty Vec as we output directly on stdout */
        Ok(Vec::<u8>::new())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

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

    #[test]
    fn test_recursive() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        
        {
            let mut tmp_file = File::create(&tmp_dir.path().join("test_file.bin")).unwrap();
            tmp_file.write(b"2tmpfile").unwrap();
        }

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&[
                "bgrep",
                "--recursive",
                "tmpfile",
                tmp_dir.path().to_str().expect("Could not convert temp path to unicode")
            ])
            .assert()
            .stdout(predicates::str::contains(": 0x1\n"))
            .success();


        
    }
}
