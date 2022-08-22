use crate::applet::Applet;
use clap::{arg, App, Command};
use std::fs;
use crate::errors::{Result, ResultExt};

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
                .chain_err(|| "Xor key decoding failed")?
        } else {
            fs::read(args.value_of("keyfile").unwrap()).chain_err(|| "Could not read keyfile")?
        };
        Ok(Box::new(Self { key_bytes }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let inf_key = self.key_bytes.iter().cycle(); // Iterate endlessly over key bytes
        return Ok(val.iter().zip(inf_key).map(|(x, k)| x ^ k).collect());
    }
}
