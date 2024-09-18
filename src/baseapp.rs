use crate::applet::{Applet, FromStrWithRadix};
use anyhow::{Context, Result};
use clap::{arg, Command};
use num_bigint::BigUint;

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
            .arg(arg!(-f --from <radix> "source radix, by default, parse standard prefixes (0x, 0b, 0o)"))
            .arg(arg!(-t --to <radix> "target radix, defaults to decimal"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let source_radix: Option<u32> = if let Some(src) = args.get_one::<String>("from") {
            Some(src.parse::<u32>().context("Could not parse 'from' radix")?)
        } else {
            None
        };
        let target_radix: u32 = if let Some(target) = args.get_one::<String>("to") {
            target.parse().context("Could not parse 'to' radix")?
        } else {
            10
        };
        Ok(Box::new(Self {
            source_radix,
            target_radix,
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let int = if let Some(src) = self.source_radix {
            BigUint::parse_bytes(&val, src).context("Could not convert input")?
        } else {
            let int_str = String::from_utf8(val).context("Could not convert value to string")?;
            BigUint::from_str_with_radix(int_str.as_str())?
        };
        Ok(int.to_str_radix(self.target_radix).as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
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
    fn test_base_cli_arg_to() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["base", "-t", "32", "0o7675"])
            .assert()
            .stdout("3tt")
            .success();
    }
}
