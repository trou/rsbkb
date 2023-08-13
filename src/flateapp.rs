use crate::applet::Applet;
use anyhow::Result;
use clap::{arg, value_parser, App, Command};
use miniz_oxide::{deflate, inflate, DataFormat};

pub struct DeflateApplet {
    format: DataFormat,
    level: u8,
}

impl Applet for DeflateApplet {
    fn command(&self) -> &'static str {
        "deflate"
    }

    fn clap_command(&self) -> App {
        Command::new(self.command())
            .about(self.description())
            .arg(
                arg!(-l --level <level> "compression level")
                    .value_parser(value_parser!(u8).range(1..11))
                    .default_value("6"),
            )
            .arg(arg!(-z --zlib "add Zlib header"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn description(&self) -> &'static str {
        "(raw) deflate compression"
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let f = if args.is_present("zlib") {
            DataFormat::Zlib
        } else {
            DataFormat::Raw
        };
        let l: &u8 = args.get_one("level").unwrap();
        Ok(Box::new(Self {
            format: f,
            level: *l,
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        match self.format {
            DataFormat::Raw => Ok(deflate::compress_to_vec(val.as_slice(), self.level)),
            _ => Ok(deflate::compress_to_vec_zlib(val.as_slice(), self.level)),
        }
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            format: DataFormat::Raw,
            level: 6,
        })
    }
}

pub struct InflateApplet {
    format: DataFormat,
    quiet: bool,
}

impl Applet for InflateApplet {
    fn command(&self) -> &'static str {
        "inflate"
    }

    fn clap_command(&self) -> App {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-z --zlib "expect Zlib header"))
            .arg(arg!(-q --quiet "don't output error message on stderr if decompression failed"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn description(&self) -> &'static str {
        "(raw) inflate decompression"
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let f = if args.is_present("zlib") {
            DataFormat::Zlib
        } else {
            DataFormat::Raw
        };
        Ok(Box::new(Self {
            format: f,
            quiet: args.is_present("quiet"),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let dec_res = match self.format {
            DataFormat::Raw => inflate::decompress_to_vec(val.as_slice()),
            _ => inflate::decompress_to_vec_zlib(val.as_slice()),
        };
        match dec_res {
            Ok(r) => Ok(r),
            Err(e) => {
                if !self.quiet {
                    eprintln!(
                        "Decompression error: {:?} (still outputing data to stdout)",
                        e.status
                    );
                }
                Ok(e.output)
            }
        }
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            format: DataFormat::Raw,
            quiet: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inflate() {
        let inf = InflateApplet {
            quiet: true,
            format: DataFormat::Raw,
        };
        assert_eq!(
            inf.process([0x2b, 0x49, 0x2d, 0x2e, 0x29, 0x01, 0x62, 0x2e, 0x00].to_vec())
                .unwrap(),
            "testtest\n".as_bytes().to_vec()
        );
    }

    #[test]
    fn test_inflate_trunc() {
        let inf = InflateApplet {
            quiet: true,
            format: DataFormat::Raw,
        };
        assert_eq!(
            inf.process([0x2b, 0x49, 0x2d, 0x2e, 0x29].to_vec())
                .unwrap(),
            [116, 101, 115, 116, 0, 0, 0, 0, 0, 0].to_vec()
        );
    }

    #[test]
    fn test_inflate_no_header() {
        let inf = InflateApplet {
            quiet: true,
            format: DataFormat::Zlib,
        };
        assert_eq!(
            inf.process([0x2b, 0x49, 0x2d, 0x2e, 0x29].to_vec())
                .unwrap(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec()
        );
    }
}
