use std::process;
use std::io::{BufReader, SeekFrom, Seek, Read};
use std::fs::{OpenOptions};
use clap::{App, SubCommand};
use crate::applet::{Applet, FromStrWithRadix};


pub struct SliceApplet {
    file :  Option<String>,
    start : u64,
    from_end : bool,
    end: Option<u64>
}

impl Applet for SliceApplet {
    fn command(&self) -> &'static str { "slice" }
    fn description(&self) -> &'static str { "slice" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
                .arg_from_usage("<file>   'file to slice'")
                .arg_from_usage("<start>  'start of slice, relative to end of file if negative'")
                .arg_from_usage("[end]  'end of slice: absolute or relative if prefixed with +'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn new() ->  Box<dyn Applet> {
        Box::new(Self { file: None, start: 0, from_end: false, end: None})
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let filename = args.value_of("file").unwrap();
        let start_val = args.value_of("start").unwrap();

        /* Negative start: offset from the end. */
        let (start, from_end) = if let Some(start_val_no_plus) = start_val.strip_prefix("-") {
                (u64::from_str_with_radix(start_val_no_plus).expect("Invalid start"), true)
        } else {
                (u64::from_str_with_radix(start_val).expect("Invalid start"), false)
        };

        let end: Option<u64> = if let Some(end_val) = args.value_of("end") {
            if let Some(end_val_no_plus) = end_val.strip_prefix("+") {
                    Some(start + u64::from_str_with_radix(end_val_no_plus).expect("Invalid end"))
            } else {
                    Some(u64::from_str_with_radix(end_val).expect("Invalid end"))
            }
        } else { None };

        Box::new(Self {file: Some(filename.to_string()), start, from_end, end })
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let start = self.start;
        let filename = self.file.as_ref().unwrap();
        let mut f =
        BufReader::new(OpenOptions::new().read(true).write(false).open(filename).expect("can't open file"));

        if self.from_end {
            f.seek(SeekFrom::End(-(self.start as i64))).expect("Seek failed");
        } else {
            f.seek(SeekFrom::Start(self.start)).expect("Seek failed");
        }
        let mut res =  vec![];
        if self.end.is_some() {
            let end = self.end.unwrap();
            if end < start {
                eprintln!("Error: specified end < start");
                process::exit(1);
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
