use std::fs;
use clap::{Arg, App, SubCommand};
use crate::applet::Applet;

pub struct XorApplet { key_bytes : Vec<u8> }

impl Applet for XorApplet {
    fn command(&self) -> &'static str { "xor" }
    fn description(&self) -> &'static str { "xor value" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
                 .arg(Arg::with_name("xorkey")
                     .short("x")
                     .takes_value(true)
                     .required_unless("keyfile")
                     .conflicts_with("keyfile")
                     .help("Xor key in hex format"))
                 .arg(Arg::with_name("keyfile")
                     .short("f")
                     .takes_value(true)
                     .required_unless("xorkey")
                     .help("File to use as key"))
                 .arg(Arg::with_name("value")
                     .required(false)
                     .help("input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { key_bytes: vec![] }) 
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let key_bytes = if args.is_present("xorkey") {
                hex::decode(args.value_of("xorkey").unwrap()).expect("Xor key decoding failed")
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
