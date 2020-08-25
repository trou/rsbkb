use crate::applet::Applet;
use crate::applet::FromStrWithRadix;
use clap::{App, SubCommand};

pub struct BofPattGenApplet {
    len : usize,
}

const UPPER : &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER : &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS : &str = "0123456789";

fn gen_pattern(len:usize, res: &mut Vec<u8>) {
        for u in UPPER.bytes() {
            if res.len() >= len {
                break;
            }
            for l in LOWER.bytes() {
                for d in DIGITS.bytes() {
                    res.push(u);
                    res.push(l);
                    res.push(d);
                }
            }
        }
}

impl Applet for BofPattGenApplet {
    fn command(&self) -> &'static str { "bofpatt" }
    fn description(&self) -> &'static str { "Buffer Overflow Pattern generator" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { len: 0})
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg_from_usage("<length> 'Pattern length'")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let max_len : usize = UPPER.len()*LOWER.len()*DIGITS.len()*3;
        let len_s = args.value_of("length").unwrap();
        let len = usize::from_str_with_radix(len_s).expect("invalid length");
        if len > max_len {
            panic!("Pattern length's too big, max is {}.", max_len); 
        }
        Box::new(Self { len })
    }

    fn process(&self, _data: Vec<u8>) -> Vec<u8> {
        let mut res : Vec<u8> = Vec::with_capacity(self.len);
        gen_pattern(self.len, &mut res);
        res.truncate(self.len);
        return res;
    }

}


pub struct BofPattOffApplet { extract: String }

impl Applet for BofPattOffApplet {
    fn command(&self) -> &'static str { "bofpattoff" }
    fn description(&self) -> &'static str { "Buffer Overflow Pattern offset finder" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { extract: String::new() })
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg_from_usage("<extract> 'Pattern extract'")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let mut extract = String::new();
        extract.push_str(args.value_of("extract").unwrap());
        Box::new(Self { extract })
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let max_len : usize = UPPER.len()*LOWER.len()*DIGITS.len()*3;
        let mut full_pattern : Vec<u8> = Vec::with_capacity(max_len);
        gen_pattern(max_len, &mut full_pattern);
        let pattern_str = String::from_utf8(full_pattern).unwrap();
        let offset = pattern_str.find(self.extract.as_str());
        let res = 
            match offset {
                Some(o) => o.to_string(),
                _ => String::from("Pattern not found")
            };
        return res.as_bytes().to_vec();
    }

}


