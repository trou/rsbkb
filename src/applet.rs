use clap::{App, SubCommand};


pub trait Applet {
    fn command(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet>;

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg_from_usage("[value] 'input value, reads from stdin in not present'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { Some("value") }

    fn process(&self, val: Vec<u8>) -> Vec<u8>;

    fn new() -> Box<dyn Applet> where Self: Sized;
}

/* Helper to trim whitespace */
pub trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b' ' || *c == b'\n' || *c == b'\r'
        }

        fn is_not_whitespace(c: &u8) -> bool {
            !is_whitespace(c)
        }

        if let Some(first) = self.iter().position(is_not_whitespace) {
            if let Some(last) = self.iter().rposition(is_not_whitespace) {
                &self[first..=last]
            } else {
                unreachable!();
            }
        } else {
            &[]
        }
    }
}

pub trait FromStrWithRadix {
        fn from_str_with_radix(s: &str)  -> Result<Self, std::num::ParseIntError> where Self:Sized;
}

impl FromStrWithRadix for u64 {
        fn from_str_with_radix(s: &str)  -> Result<u64, std::num::ParseIntError> {
            if s.len() > 2 && &s[0..2] == "0x" {
                return u64::from_str_radix(&s[2..], 16);
            } else {
                return s.parse();
            }
        }
    }

impl FromStrWithRadix for i64 {
        fn from_str_with_radix(s: &str)  -> Result<i64, std::num::ParseIntError> {
            if s.len() > 2 && &s[0..2] == "0x" {
                return i64::from_str_radix(&s[2..], 16);
            } else {
                return s.parse();
            }
        }
    }

impl FromStrWithRadix for usize {
        fn from_str_with_radix(s: &str)  -> Result<usize, std::num::ParseIntError> {
            if s.len() > 2 && &s[0..2] == "0x" {
                return usize::from_str_radix(&s[2..], 16);
            } else {
                return s.parse();
            }
        }
    }
