#![allow(clippy::new_ret_no_self)]
use anyhow::{Context, Result};
use clap::{arg, Command};

pub trait Applet {
    /// The string which will define the subcommand.
    fn command(&self) -> &'static str;

    /// Simple description of the applet.
    fn description(&self) -> &'static str;

    /// Overload to return "false" if the applet directly
    /// outputs data to stdout.
    fn returns_data(&self) -> bool {
        true
    }

    /// Receives the arguments as understood by `clap` and builds the resulting `Applet`.
    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>>;

    /// Define the applet's arguments.
    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!([value] "input value, reads from stdin if not present"))
    }

    /// By default, applets accept the input as:
    ///   - an argument, named "value"
    ///   - stdin, if value is not supplied
    ///
    /// Applets can overload this method to have a different behaviour
    /// (for example if they have more args).
    fn arg_or_stdin(&self) -> Option<&'static str> {
        Some("value")
    }

    /// Called by `main` to process the data in `val`
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

/* Helper to trim whitespace
 * Note: trim_ascii does this in Rust 1.80
 * */
pub trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        const fn is_not_whitespace(c: &u8) -> bool {
            !c.is_ascii_whitespace()
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

// We cannot use a default implementation for trait as from_str_radix is not defined
// in a trait but as a function of for most base types
pub trait FromStrWithRadix {
    fn from_str_with_radix(s: &str) -> Result<Self>
    where
        Self: Sized;
}

// Helper to add "from_str_with_radix" to given types
macro_rules! from_str_with_radix_for_types {
    ($($x:ident),* )  =>
        {
$(
impl FromStrWithRadix for $x {
    fn from_str_with_radix(s: &str) -> Result<Self> {
        if s.len() > 2 && &s[0..2] == "0x" {
            Self::from_str_radix(&s[2..], 16)
        } else if s.len() > 2 && &s[0..2] == "0o" {
            Self::from_str_radix(&s[2..], 8)
        } else {
            s.parse()
        }
        .context("Could not convert str")
    }
})*

        };
}

from_str_with_radix_for_types!(u64, i64, usize);
