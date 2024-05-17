use crate::applet::Applet;
use crate::applet::SliceExt;
use anyhow::Result;
use clap::{arg, Command};

pub struct UrlEncApplet {
    excluded: String,
}

impl Applet for UrlEncApplet {
    fn command(&self) -> &'static str {
        "urlenc"
    }
    fn description(&self) -> &'static str {
        "URL encode"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            excluded: "".to_string(),
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-e --"exclude-chars" <chars>  "a string of chars to exclude from encoding"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
            .after_help("By default, encode all non alphanumeric characters in the input.")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        if args.contains_id("exclude-chars") {
            let chars: &String = args.get_one::<String>("exclude-chars").unwrap();

            Ok(Box::new(Self {
                excluded: chars.to_string(),
            }))
        } else {
            Ok(Box::new(Self {
                excluded: "".to_string(),
            }))
        }
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let mut table = [false; 256];

        for i in 0..255 {
            let c = char::from_u32(i).unwrap();
            if !c.is_ascii_graphic() {
                table[i as usize] = true;
            } else {
                if matches!(
                    c,
                    '!' | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '\''
                        | '('
                        | ')'
                        | '*'
                        | '+'
                        | ','
                        | '/'
                        | ':'
                        | ';'
                        | '='
                        | '?'
                        | '@'
                        | '['
                        | ']'
                ) && !self.excluded.contains(c)
                {
                    table[i as usize] = true;
                }
            }
        }
        let mut encoded = Vec::with_capacity(val.len());
        for b in val.iter() {
            if table[*b as usize] {
                encoded.extend_from_slice(format!("%{:02x}", *b).as_bytes());
            } else {
                encoded.push(*b);
            };
        }
        Ok(encoded)
    }
}

pub struct UrlDecApplet {}

impl Applet for UrlDecApplet {
    fn command(&self) -> &'static str {
        "urldec"
    }
    fn description(&self) -> &'static str {
        "URL decode"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, urlval: Vec<u8>) -> Result<Vec<u8>> {
        let trimmed: Vec<u8> = urlval.trim().into();
        let decoded: Vec<u8> = percent_encoding::percent_decode(&trimmed).collect();
        Ok(decoded)
    }
}
