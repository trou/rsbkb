use crate::applet::{Applet, FromStrWithRadix};
use anyhow::{bail, Context, Result};
use clap::{arg, Command};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

pub struct SliceApplet {
    file: Option<String>,
    start: u64,
    from_end: bool,
    end: Option<u64>,
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
            .arg(arg!([end]   "end of slice: absolute or relative if prefixed with +"))
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            file: None,
            start: 0,
            from_end: false,
            end: None,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let filename = args.get_one::<String>("file").unwrap();
        let start_val = args.get_one::<String>("start").unwrap();

        /* Negative start: offset from the end. */
        let (start, from_end) = if let Some(start_val_no_plus) = start_val.strip_prefix('-') {
            (
                u64::from_str_with_radix(start_val_no_plus)
                    .with_context(|| "Invalid value for 'start'")?,
                true,
            )
        } else {
            (
                u64::from_str_with_radix(start_val).with_context(|| "Invalid value for 'start'")?,
                false,
            )
        };

        let end: Option<u64> = if let Some(end_val) = args.get_one::<String>("end") {
            if let Some(end_val_no_plus) = end_val.strip_prefix('+') {
                Some(
                    start
                        + u64::from_str_with_radix(end_val_no_plus)
                            .with_context(|| "Invalid end")?,
                )
            } else {
                Some(u64::from_str_with_radix(end_val).with_context(|| "Invalid end")?)
            }
        } else {
            None
        };

        Ok(Box::new(Self {
            file: Some(filename.to_string()),
            start,
            from_end,
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
            if self.from_end {
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
        let mut res = Vec::with_capacity(self.start as usize);

        f.read_exact(&mut res)
            .with_context(|| "Could not read until start")?;

        // Drop it
        res.clear();

        if self.end.is_some() {
            let end = self.end.unwrap();
            if end < self.start {
                bail!("specified end < start");
            }
            let len: usize = (end - self.start) as usize;
            res.resize(len, 0);
            f.read_exact(&mut res).with_context(|| "Read failed")?;
        } else {
            f.read_to_end(&mut res).with_context(|| "Read failed")?;
        }
        Ok(res.to_vec())
    }

    fn process_seekable(&self, filename: &str) -> Result<Vec<u8>> {
        let mut f = BufReader::new(
            OpenOptions::new()
                .read(true)
                .write(false)
                .open(filename)
                .with_context(|| format!("can't open file \"{}\"", filename))?,
        );

        if self.from_end {
            f.seek(SeekFrom::End(-(self.start as i64)))
                .with_context(|| "seek failed")?;
        } else {
            f.seek(SeekFrom::Start(self.start))
                .with_context(|| "seek failed")?;
        }
        let mut res = vec![];
        if self.end.is_some() {
            let end = self.end.unwrap();
            if end < self.start {
                bail!("specified end < start");
            }
            let len: usize = (end - self.start) as usize;
            res.resize(len, 0);
            f.read_exact(&mut res).with_context(|| "Read failed")?;
        } else {
            f.read_to_end(&mut res).with_context(|| "Read failed")?;
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
            end: Some(0),
            start: 0,
            from_end: false,
        };

        assert_eq!(d[0..0], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            end: Some(10),
            start: 0,
            from_end: false,
        };

        assert_eq!(d[0..10], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice_to_end() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            end: None,
            start: 10,
            from_end: false,
        };

        assert_eq!(d[10..], pat.process_test(Vec::new()));
    }

    #[test]
    fn test_slice_from_end() {
        let (tmpfile, d) = setup();
        let filepath = tmpfile.path().to_str().unwrap().to_string();
        let pat = SliceApplet {
            file: Some(filepath),
            end: None,
            start: 10,
            from_end: true,
        };

        assert_eq!(d[(d.len() - 10)..], pat.process_test(Vec::new()));
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
            .write_stdin(&data)
            .assert()
            .stdout(&b"\x02\x03\x04"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "2"])
            .write_stdin(&data)
            .assert()
            .stdout(&b"\x02\x03\x04\x05\x06\x07\x08\x09"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["slice", "-", "-2"])
            .write_stdin(&data)
            .assert()
            .stdout("")
            .failure();
    }
}
