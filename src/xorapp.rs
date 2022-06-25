use std::fs;
use clap::{Arg, App, SubCommand};
use crate::applet::Applet;

pub struct XorApplet { key_bytes : Vec<u8> }

impl Applet for XorApplet {
    fn command(&self) -> &'static str { "xor" }
    fn description(&self) -> &'static str { "xor value" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg(Arg::from_usage("-x --xorkey=[KEY] 'Xor key in hex format'")
                     .required_unless("keyfile")
                     .conflicts_with("keyfile"))
             .arg_from_usage("-f --keyfile=[keyfile] 'File to use as key'")
             .arg_from_usage("[value] 'input value, reads from stdin in not present'")
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { key_bytes: vec![] })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let key_bytes = if args.is_present("xorkey") {
                hex::decode(args.value_of("xorkey").unwrap().replace(' ',"")).expect("Xor key decoding failed")
            } else {
                fs::read(args.value_of("keyfile").unwrap()).expect("can't open keyfile")
            };
        Box::new(Self { key_bytes } )
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        let inf_key = self.key_bytes.iter().cycle(); // Iterate endlessly over key bytes
        return val.iter().zip(inf_key).map (|(x, k)| x ^ k).collect();
    }

}
