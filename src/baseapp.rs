use crate::applet::Applet;
use anyhow::{Context, Result};
use clap::{arg, Command};
use num_bigint::BigUint;
use num_traits::Num;

use crate::applet::SliceExt;

pub struct BaseIntApplet {
    source_radix: Option<u32>,
    target_radix: u32,
}

impl Applet for BaseIntApplet {
    fn command(&self) -> &'static str {
        "base"
    }
    fn description(&self) -> &'static str {
        "convert integer between different bases"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            source_radix: None,
            target_radix: 10,
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-f --from <radix> "source radix, by default, parse standard prefixes (0x, 0b, 0o)")
                 .value_parser(clap::value_parser!(u32).range(2..37)))
            .arg(arg!(-t --to <radix> "target radix, defaults to decimal, except if input was decimal, then default to hex")
                 .value_parser(clap::value_parser!(u32).range(2..37)))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let target_radix: u32 = if let Some(target) = args.get_one::<u32>("to") {
            *target
        } else {
            10
        };
        Ok(Box::new(Self {
            source_radix: args.get_one::<u32>("from").copied(),
            target_radix,
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        // Remove whitespace to make conversions work with stdin input
        let val = val.trim();

        let (srcrad, int) = if let Some(src) = self.source_radix {
            (
                src,
                BigUint::parse_bytes(val, src).context("Could not convert input")?,
            )
        } else {
            let int_str = String::from_utf8_lossy(val);

            // Base was not specified, check standard prefixes
            if int_str.len() > 2 && &int_str[0..2] == "0x" {
                (
                    16,
                    BigUint::from_str_radix(&int_str[2..], 16)
                        .context("Could not parse argument as hex integer")?,
                )
            } else if int_str.len() > 2 && &int_str[0..2] == "0o" {
                (
                    8,
                    BigUint::from_str_radix(&int_str[2..], 8)
                        .context("Could not parse argument as octal integer")?,
                )
            } else {
                (
                    10,
                    int_str
                        .parse()
                        .context("Could not parse argument as integer")?,
                )
            }
        };

        // If both source and target radices are equal to 10, actually output hex
        if srcrad == 10 && self.target_radix == 10 {
            Ok((format!("0x{}", int.to_str_radix(16))).as_bytes().to_vec())
        } else {
            Ok(int.to_str_radix(self.target_radix).as_bytes().to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_base_cli_no_radix() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base", "10"])
            .assert()
            .stdout("0xa")
            .success();
    }

    #[test]
    fn test_base_cli_arg() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base", "0x10"])
            .assert()
            .stdout("16")
            .success();
    }

    #[test]
    fn test_base_cli_arg_from_to() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base", "-f", "2", "-t", "16", "10000"])
            .assert()
            .stdout("10")
            .success();
    }

    #[test]
    fn test_base_cli_stdin() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base"])
            .write_stdin("0xA\n")
            .assert()
            .stdout("10")
            .success();
    }

    #[test]
    fn test_base_cli_arg_to() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base", "-t", "32", "0o7675"])
            .assert()
            .stdout("3tt")
            .success();
    }
}
