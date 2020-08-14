use std::io::{BufReader, SeekFrom, Seek, Read};
use std::fs::{OpenOptions};
use clap::{App, SubCommand};
use crate::applet::Applet;

fn num_from_str_safe(s: &str) -> Result<u64, std::num::ParseIntError> {
    if s.len() > 2 && &s[0..2] == "0x" {
        return u64::from_str_radix(&s[2..], 16);
    } else {
        return s.parse();
    }
}

pub struct SliceApplet {
    file :  Option<String>,
    start : u64,
    end: Option<u64>
}

impl Applet for SliceApplet {
    fn command(&self) -> &'static str { "slice" }
    fn description(&self) -> &'static str { "slice" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
                .arg_from_usage("<file>   'file to slice'")
                .arg_from_usage("<start>  'start of slice'")
                .arg_from_usage("[end]  'end of slice: absolute or relative if prefixed with +'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn new() ->  Box<dyn Applet> {
        Box::new(Self { file: None, start: 0, end: None})
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let filename = args.value_of("file").unwrap();
        let start_va = args.value_of("start").unwrap();
        let start: u64 = num_from_str_safe(start_va).expect("invalid start");

        let end: Option<u64> = if let Some(end_val) = args.value_of("end") {
            if end_val.starts_with("+") {
                    Some(start + num_from_str_safe(&end_val[1..]).expect("Invalid end"))
                } else {
                    Some(num_from_str_safe(end_val).expect("Invalid end"))
                }
            } else { None };

        Box::new(Self {file: Some(filename.to_string()), start, end })
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let start = self.start;
        let filename = self.file.as_ref().unwrap();
        let mut f =
        BufReader::new(OpenOptions::new().read(true).write(false).open(filename).expect("can't open file"));

        f.seek(SeekFrom::Start(self.start)).expect("Seek failed");
        let mut res =  vec![];
        if self.end.is_some() {
            let end = self.end.unwrap();
            if end < start {
                panic!("end < start");
            }
            let len: usize = (end-start) as usize;
            res.resize(len, 0);
            f.read_exact(&mut res).expect("Read failed");
        } else {
            f.read_to_end(&mut res).expect("Read failed");
        }
        return res.to_vec();
    }

}
