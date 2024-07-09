use crate::applet::{Applet, FromStrWithRadix};
use anyhow::{bail, Context, Result};
use clap::{arg, Command};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

#[derive(Debug)]
struct Position {
    offset: u64,
    relative: bool,
    from_end: bool,
}

pub struct SliceApplet {
    file: Option<String>,
    start: Position,
    end: Option<Position>,
}

/* Helper to parse "start" and "end".
 * return value: value, plus_prefix, minus_prefix
 */
fn parse_value_with_prefix(s: &String) -> Result<Position> {
    if s.len() < 1 {
        bail!("Invalid length for value");
    }

    let first = s.chars().nth(0).unwrap();

    let str_stripped = &s[1..].to_string();

    let (from_end, relative, str_strip): (bool, bool, &String) = if first == '-' {
        (true, false, str_stripped)
    } else if first == '+' {
        (false, true, str_stripped)
    } else {
        (false, false, s)
    };
    let offset = u64::from_str_with_radix(&str_strip).with_context(|| "Invalid offset value")?;
    Ok(Position {
        offset,
        relative,
        from_end,
    })
}

impl Applet for SliceApplet {
    fn command(&self) -> &'static str {
        "slice"
    }
    fn description(&self) -> &'static str {
        "slice"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(<file>    "file to slice, - for stdin"))
            .arg(arg!(<start>   "start of slice, relative to end of file if negative"))
            .arg(arg!([end]     "end of slice: absolute, relative to <start> if prefixed with +, relative to end of file if negative"))
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            file: None,
            start: Position {
                offset: 0,
                relative: false,
                from_end: false,
            },
            end: None,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let filename = args.get_one::<String>("file").unwrap();
        let start_val = args.get_one::<String>("start").unwrap();
        let end_opt = args.get_one::<String>("end");

        let start = parse_value_with_prefix(start_val)?;

        let end = match end_opt {
            None => None,
            Some(end_val) => Some(parse_value_with_prefix(end_val)?),
        };

        Ok(Box::new(Self {
            file: Some(filename.to_string()),
            start,
            end,
        }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let filename = self.file.as_ref().unwrap();

        if filename == "-"
            || File::open(filename)
                .with_context(|| format!("can't open file \"{}\"", filename))?
                .rewind()
                .is_err()
        {
            if self.start.from_end || self.end.as_ref().is_some_and(|e| e.from_end) {
                bail!("Cannot seek from end in an unseekable file");
            }
            self.process_unseekable(filename)
        } else {
            self.process_seekable(filename)
        }
    }
}

impl SliceApplet {
    fn process_unseekable(&self, filename: &str) -> Result<Vec<u8>> {
        let mut f: Box<dyn BufRead> = if filename == "-" {
            Box::new(BufReader::new(std::io::stdin()))
        } else {
            Box::new(BufReader::new(
                OpenOptions::new()
                    .read(true)
                    .write(false)
                    .open(filename)
                    .with_context(|| format!("can't open file \"{}\"", filename))?,
            ))
        };

        // Read initial data
        let mut res = vec![0; self.start.offset as usize];

        f.read_exact(&mut res)
            .with_context(|| "Could not read until start")?;

        // Drop it
        res.clear();

        let start = self.start.offset;

        if self.end.is_some() {
            let end_pos = self.end.as_ref().unwrap();

            let end = if end_pos.relative {
                start + end_pos.offset
            } else {
                end_pos.offset
            };

            if end < start {
                bail!("specified end < start");
            }
            let len: usize = (end - start) as usize;
            res.resize(len, 0);
            f.read_exact(&mut res).with_context(|| "Read failed")?;
        } else {
            f.read_to_end(&mut res).with_context(|| "Read failed")?;
        }
        Ok(res.to_vec())
    }

    fn process_seekable(&self, filename: &str) -> Result<Vec<u8>> {
        let f = OpenOptions::new()
            .read(true)
            .write(false)
            .open(filename)
            .with_context(|| format!("can't open file \"{}\"", filename))?;
        let flen = f
            .metadata()
            .with_context(|| format!("Could not get file len for {}", filename))?
            .len();
        let mut fbuf = BufReader::new(&f);

        let start = if self.start.from_end {
            if self.start.offset > flen {
                    bail!("start is before beginning of file");
            }
            flen - self.start.offset
        } else {
            self.start.offset
        };
        if start > flen {
            bail!("start (0x{:X}) is after end of file (0x{:X})", start, flen);
        }
        fbuf.seek(SeekFrom::Start(start))
            .with_context(|| "seek failed")?;

        let mut res = vec![];
        if self.end.is_some() {
            let end_pos = self.end.as_ref().unwrap();
            let end = if end_pos.from_end {
                flen - end_pos.offset
            } else if end_pos.relative {
                start + end_pos.offset
            } else {
                end_pos.offset
            };

            if end < start {
                bail!("specified end < start");
            } else if end > flen {
                bail!("end (0x{:X}) is after end of file (0x{:X})", end, flen);
            }
            let len: usize = (end - start) as usize;
            res.resize(len, 0);
            fbuf.read_exact(&mut res).with_context(|| "Read failed")?;
        } else {
            fbuf.read_to_end(&mut res).with_context(|| "Read failed")?;
        }
        Ok(res.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn setup() -> (NamedTempFile, [u8; 100]) {
        let mut rand_data = [0u8; 100];
        thread_rng().fill(&mut rand_data[..]);

        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        tmpfile.write(&rand_data).unwrap();
        (tmpfile, rand_data)
    }

    #[test]
    fn test_empty_slice() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            start: Position {
                offset: 0,
                relative: false,
                from_end: false,
            },
            end: Some(Position {
                offset: 0,
                relative: false,
                from_end: false,
            }),
        };

        assert_eq!(d[0..0], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            start: Position {
                offset: 0,
                relative: false,
                from_end: false,
            },
            end: Some(Position {
                offset: 10,
                relative: false,
                from_end: false,
            }),
        };

        assert_eq!(d[0..10], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice_to_end() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            start: Position {
                offset: 10,
                relative: false,
                from_end: false,
            },
            end: None,
        };

        assert_eq!(d[10..], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice_end_from_end() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            start: Position {
                offset: 10,
                relative: false,
                from_end: false,
            },
            end: Some(Position {
                offset: 10,
                relative: false,
                from_end: true,
            }),
        };

        assert_eq!(d[10..(d.len() - 10)], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_cli_file() {
        let mut data: [u8; 10] = [0; 10];
        for i in (0..10).into_iter() {
            data[i] = i as u8;
        }

        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        tmpfile.write(&data).unwrap();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", &tmpfile.path().to_str().unwrap(), "2", "+0x3"])
            .assert()
            .stdout(&b"\x02\x03\x04"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", &tmpfile.path().to_str().unwrap(), "2"])
            .assert()
            .stdout(&b"\x02\x03\x04\x05\x06\x07\x08\x09"[..])
            .success();

        /* Should fail because "start" is before beginning of file */
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", &tmpfile.path().to_str().unwrap(), "-200"])
            .assert()
            .failure();

        /* Should fail because "end" is before "start */
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", &tmpfile.path().to_str().unwrap(), "0", "-300"])
            .assert()
            .failure();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "--", &tmpfile.path().to_str().unwrap(), "-2"])
            .assert()
            .stdout(&b"\x08\x09"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&[
                "slice",
                "--",
                &tmpfile.path().to_str().unwrap(),
                "-0x2",
                "+1",
            ])
            .assert()
            .stdout(&b"\x08"[..])
            .success();
    }

    #[test]
    fn test_cli_stdin() {
        let mut data: [u8; 10] = [0; 10];
        for i in (0..10).into_iter() {
            data[i] = i as u8;
        }

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "2", "+3"])
            .write_stdin(*&data)
            .assert()
            .stdout(&b"\x02\x03\x04"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "2"])
            .write_stdin(*&data)
            .assert()
            .stdout(&b"\x02\x03\x04\x05\x06\x07\x08\x09"[..])
            .success();

        /* Should fail because stdin is not seekable */
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "-2"])
            .write_stdin(*&data)
            .assert()
            .stdout("")
            .failure();

        /* Should fail because stdin is not seekable */
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "0", "-10"])
            .write_stdin(*&data)
            .assert()
            .stdout("")
            .failure();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "0", "0"])
            .write_stdin(*&data)
            .assert()
            .stdout(&b""[..])
            .success();
    }
}
