use crate::applet::Applet;
use crate::applet::FromStrWithRadix;
use anyhow::Result;
use clap::{arg, Command};
use std::char;

pub struct BofPattGenApplet {
    len: usize,
}

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";

fn gen_pattern(len: usize, res: &mut Vec<u8>) {
    for u in UPPER.bytes() {
        if res.len() >= len {
            return;
        }
        for l in LOWER.bytes() {
            for d in DIGITS.bytes() {
                res.push(u);
                res.push(l);
                res.push(d);
            }
        }
    }
    gen_pattern(len - res.len(), res);
}

impl Applet for BofPattGenApplet {
    fn command(&self) -> &'static str {
        "bofpatt"
    }
    fn description(&self) -> &'static str {
        "buffer overflow pattern generator"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { len: 0 })
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(<length>  "Pattern length"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let max_len: usize = UPPER.len() * LOWER.len() * DIGITS.len() * 3;
        let len_s = args.get_one::<String>("length").unwrap();
        let len = usize::from_str_with_radix(len_s)?;
        if len > max_len {
            eprintln!("Warning: pattern length's longer than max_len {}.", max_len);
        }
        Ok(Box::new(Self { len }))
    }

    fn process(&self, _data: Vec<u8>) -> Result<Vec<u8>> {
        let mut res: Vec<u8> = Vec::with_capacity(self.len);
        gen_pattern(self.len, &mut res);
        res.truncate(self.len);
        Ok(res)
    }
}

pub struct BofPattOffApplet {
    extract: String,
}

impl Applet for BofPattOffApplet {
    fn command(&self) -> &'static str {
        "bofpattoff"
    }
    fn description(&self) -> &'static str {
        "buffer overflow pattern offset finder"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            extract: String::new(),
        })
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-b --"big-endian"  "Parse hex value as big endian"))
            .arg(arg!(<extract>  "Pattern extract (Use 0xAABBCCDD for reg value)"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let mut extract = String::new();
        let arg_val = args.get_one::<String>("extract").unwrap();
        let big_endian = args.get_flag("big-endian");
        if &arg_val[0..2] == "0x" {
            let mut arg_int = u64::from_str_with_radix(arg_val)?;
            while arg_int != 0 {
                let c = char::from_u32((arg_int & 0xFF) as u32).unwrap();
                if big_endian {
                    extract.insert(0, c);
                } else {
                    extract.push(c);
                }
                arg_int >>= 8;
            }
            println!("Decoded pattern: {} (big endian: {})", extract, big_endian);
        } else {
            extract.push_str(arg_val);
        }
        Ok(Box::new(Self { extract }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let max_len: usize = UPPER.len() * LOWER.len() * DIGITS.len() * 3;
        let mut full_pattern: Vec<u8> = Vec::with_capacity(max_len);
        gen_pattern(max_len, &mut full_pattern);
        let pattern_str = String::from_utf8(full_pattern).unwrap();
        let offset = pattern_str.find(self.extract.as_str());
        let res = match offset {
            Some(o) => format!("Offset: {} (mod {}) / {:#x}", o, max_len, o),
            _ => String::from("Pattern not found"),
        };
        Ok(res.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen() {
        let pat = BofPattGenApplet { len: 40 };
        assert_eq!(
            String::from_utf8(pat.process_test(vec![])).unwrap(),
            "Aa0Aa1Aa2Aa3Aa4Aa5Aa6Aa7Aa8Aa9Ab0Ab1Ab2A"
        );
    }

    #[test]
    fn test_off() {
        let pat = BofPattOffApplet {
            extract: String::from("Yq6Y"),
        };
        assert_eq!(
            String::from_utf8(pat.process_test(vec![])).unwrap(),
            "Offset: 19218 (mod 20280) / 0x4b12"
        );
    }

    #[test]
    fn test_not_found() {
        let pat = BofPattOffApplet {
            extract: String::from("***"),
        };
        assert_eq!(
            String::from_utf8(pat.process_test(vec![])).unwrap(),
            "Pattern not found"
        );
    }
}
