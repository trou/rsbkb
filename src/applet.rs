use crate::errors::{Result, ResultExt};
use clap::{arg, App, Command};

pub trait Applet {
    fn command(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>>;

    fn clap_command(&self) -> App {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!([value] "input value, reads from stdin in not present"))
    }

    /* By default, applets accept the input as:
     *   - an argument, named "value"
     *   - stdin, if value is not supplied
     * Applets can overload this method to have a different behaviour
     * (for example if they have more args
     * */
    fn arg_or_stdin(&self) -> Option<&'static str> {
        Some("value")
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>>;

    /* No error wrapping to make it easier to test */
    #[cfg(test)]
    fn process_test(&self, val: Vec<u8>) -> Vec<u8> {
        self.process(val).unwrap()
    }

    fn new() -> Box<dyn Applet>
    where
        Self: Sized;
}

/* Helper to trim whitespace */
pub trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        const fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b' ' || *c == b'\n' || *c == b'\r'
        }

        const fn is_not_whitespace(c: &u8) -> bool {
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
    fn from_str_with_radix(s: &str) -> Result<Self>
    where
        Self: Sized;
}

impl FromStrWithRadix for u64 {
    fn from_str_with_radix(s: &str) -> Result<Self> {
        if s.len() > 2 && &s[0..2] == "0x" {
            Self::from_str_radix(&s[2..], 16)
        } else {
            s.parse()
        }
        .chain_err(|| "Could not convert str")
    }
}

impl FromStrWithRadix for i64 {
    fn from_str_with_radix(s: &str) -> Result<Self> {
        if s.len() > 2 && &s[0..2] == "0x" {
            Self::from_str_radix(&s[2..], 16)
        } else {
            s.parse()
        }
        .chain_err(|| "Could not convert str")
    }
}

impl FromStrWithRadix for usize {
    fn from_str_with_radix(s: &str) -> Result<Self> {
        if s.len() > 2 && &s[0..2] == "0x" {
            Self::from_str_radix(&s[2..], 16)
        } else {
            s.parse()
        }
        .chain_err(|| "Could not convert str")
    }
}
