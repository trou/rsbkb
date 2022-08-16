use clap::{App, SubCommand};
use crate::applet::Applet;
use elf_utilities::{file::ELF, section::Contents64, section::Contents32};


pub struct FindSoApplet {
    function :  Option<String>,
    files : Option<Vec<String>>
}

impl Applet for FindSoApplet {
    fn command(&self) -> &'static str { "findso" }
    fn description(&self) -> &'static str { "findso" }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
                .arg_from_usage("<function> 'function to search'")
                .arg_from_usage("<files>... 'files to search in'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { None }

    fn new() ->  Box<dyn Applet> {
        Box::new(Self { files: None, function: None})
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        let filenames : Vec<String> = args.values_of("files").unwrap().map(|x| x.to_string()).collect();
        let function_val = args.value_of("function").unwrap();


        Box::new(Self {files: Some(filenames), function: Some(function_val.to_string())})
    }

    fn process(&self, _val: Vec<u8>) -> Vec<u8> {
        let fun = self.function.as_ref().unwrap();
        let sofiles = self.files.as_ref().unwrap();
        for f in sofiles.iter() {
            let elf_file = elf_utilities::parser::parse_elf(f.as_str()).unwrap();
            /* For each file, iterate over sections and for "Symbols" section,
             * iterate over symbols to check if one matches "fun" */
            let found: bool = match elf_file {
                    |  ELF::ELF64(f) => {
                            f.sections.iter().map(|s| {
                                match &s.contents {
                                    | Contents64::Symbols(syms) => { syms.iter().find(|s| &s.symbol_name == fun).is_some() }
                                    | _ => { false }
                                }
                            }, ).find(|x| x == &true).is_some()
                        }
                    |  ELF::ELF32(f) => {
                            f.sections.iter().map(|s| {
                                match &s.contents {
                                    | Contents32::Symbols(syms) => { syms.iter().find(|s| &s.symbol_name == fun).is_some() }
                                    | _ => { false }
                                }
                            }, ).find(|x| x == &true).is_some()
                        }
            };
            if found {
                print!("{}", f);
            }
        }
        /* Return empty Vec as we output directly on stdout */
        return Vec::<u8>::new();
    }

}
