use clap::{App, SubCommand};
use std::fs;
use crate::applet::Applet;
use goblin::elf;
use std::str::FromStr;


pub struct FindSoApplet {
    function :  Option<String>,
    files : Option<Vec<String>>,
    is_ref :  bool,
}

impl Applet for FindSoApplet {
    fn command(&self) -> &'static str { "findso" }
    fn description(&self) -> &'static str { "findso" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
                .arg_from_usage("-r 'use first file as reference ELF to get .so list from'")
                .arg_from_usage("<function> 'function to search'")
                .arg_from_usage("<files>... 'files to search in'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn new() ->  Box<dyn Applet> {
        Box::new(Self { files: None, function: None, is_ref: false})

    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let filenames : Vec<String> = args.values_of("files").unwrap().map(|x| x.to_string()).collect();
        let function_val = args.value_of("function").unwrap();


        Box::new(Self {files: Some(filenames), function: Some(function_val.to_string()),
                       is_ref: args.is_present("r")})
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let fun = self.function.as_ref().unwrap();
        let mut sofiles : Vec<String> = Vec::from(self.files.as_ref().unwrap().as_slice());
        if self.is_ref {
            let f_data :Vec<u8> = fs::read(self.files.as_ref().unwrap()[0].as_str()).expect("Could not read file");
            let elf_ref = elf::Elf::parse(f_data.as_slice()).unwrap();
            sofiles.extend(elf_ref.libraries.iter().map(|l| String::from_str(l).unwrap()));
        };
        println!("{:?}", sofiles);
        for f in sofiles.iter() {
            let f_data :Vec<u8> = fs::read(f).expect("Could not read file");
            let elf_file = elf::Elf::parse(f_data.as_slice()).unwrap();
            let strtab = elf_file.dynstrtab;

            let found = elf_file
                .dynsyms
                .iter()
                .find( |s| { !s.is_import() && strtab.get_at(s.st_name) == Some(fun) }).is_some();
            if found {
                print!("{}", f);
            }
        }
        /* Return empty Vec as we output directly on stdout */
        return Vec::<u8>::new();
    }

}
