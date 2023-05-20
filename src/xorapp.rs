use crate::applet::Applet;
use anyhow::{Context, Result};
use clap::{arg, App, Command};
use std::fs;

pub struct XorApplet {
    key_bytes: Vec<u8>,
}

impl Applet for XorApplet {
    fn command(&self) -> &'static str {
        "xor"
    }
    fn description(&self) -> &'static str {
        "xor value"
    }

    fn clap_command(&self) -> App {
        Command::new(self.command())
            .about(self.description())
            .arg(
                arg!(-x --xorkey [KEY]  "Xor key in hex format")
                    .required_unless("keyfile")
                    .conflicts_with("keyfile"),
            )
            .arg(arg!(-f --keyfile [keyfile]  "File to use as key"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { key_bytes: vec![] })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let key_bytes = if args.is_present("xorkey") {
            hex::decode(args.value_of("xorkey").unwrap().replace(' ', ""))
                .with_context(|| "Xor key decoding failed")?
        } else {
            fs::read(args.value_of("keyfile").unwrap()).with_context(|| "Could not read keyfile")?
        };
        Ok(Box::new(Self { key_bytes }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let inf_key = self.key_bytes.iter().cycle(); // Iterate endlessly over key bytes
        return Ok(val.iter().zip(inf_key).map(|(x, k)| x ^ k).collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let data = vec![1, 0x55, 0xAA, 0xFF, 0];
        let x = XorApplet {
            key_bytes: data.clone(),
        };
        assert_eq!(x.process_test(vec![0, 0, 0, 0, 0]), data);
        assert_eq!(
            x.process_test(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            vec![0xFE, 0xAA, 0x55, 0, 0xFF]
        );
        assert_eq!(x.process_test(vec![0]), vec![1]);
        assert_eq!(
            x.process_test(vec![0, 0, 0, 0, 0, 0]),
            vec![1, 0x55, 0xAA, 0xFF, 0, 1]
        );
    }
}
