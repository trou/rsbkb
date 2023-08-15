use crate::applet::Applet;
use anyhow::{Context, Result};
use clap::{arg, Command};
use goblin::elf;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub struct FindSoApplet {
    // Function we are looking for
    function: Option<String>,
    // .so files
    files: Option<Vec<String>>,
    // First .so is a binary to look for dependencies in
    is_ref: bool,
    // LD_LIBRARY_PATH equivalent
    paths: Option<Vec<PathBuf>>,
}

impl Applet for FindSoApplet {
    fn command(&self) -> &'static str {
        "findso"
    }

    fn description(&self) -> &'static str {
        "find which .so implements a given function"
    }

    fn returns_data(&self) -> bool {
        false
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-r --ref  "use first file as reference ELF to get .so list from"))
            .arg(arg!(-p --ldpath <LDPATH> "'\':\' separated list of paths to look for .so in'"))
            .arg(arg!(-l --ldconf <CONF>  "use config file to get LD paths"))
            .arg(arg!(<function>  "function to search"))
            .arg(arg!(<files>...  "files to search in"))
    }

    fn arg_or_stdin(&self) -> Option<&'static str> {
        None
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            files: None,
            function: None,
            is_ref: false,
            paths: None,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let filenames: Vec<String> = args
            .get_many::<String>("files")
            .unwrap()
            .map(|x| x.to_string())
            .collect();
        let function_val = args.get_one::<String>("function").unwrap();
        let paths = if args.contains_id("ldpath") || args.contains_id("ldconf") {
            let mut paths: Vec<PathBuf> = if let Some(ldpaths) = args.get_one::<String>("ldpath") {
                ldpaths
                    .split(':')
                    .map(|p| PathBuf::from_str(p).unwrap())
                    .collect()
            } else {
                Vec::<PathBuf>::new()
            };

            // parse ld.so.conf "like" file
            if args.contains_id("ldconf") {
                let ldconf = args.get_one::<String>("ldconf").unwrap();
                let ldpaths: Vec<PathBuf> = fs::read_to_string(ldconf)
                    .with_context(|| format!("Could not read config file \"{}\"", ldconf))?
                    .split('\n')
                    .filter(|p| p.get(0..1).unwrap_or("#") != "#") // Skip empty lines and comments
                    .map(|p| PathBuf::from_str(p).unwrap())
                    .collect::<Vec<PathBuf>>();
                paths.extend(ldpaths);
            }

            // check that paths are actually valid directories
            for p in paths.iter() {
                if !p.is_dir() {
                    eprintln!("Warning: {} is not a valid directory", p.to_str().unwrap());
                }
            }

            Some(paths)
        } else {
            None
        };

        Ok(Box::new(Self {
            files: Some(filenames),
            function: Some(function_val.to_string()),
            is_ref: args.contains_id("ref"),
            paths,
        }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let fun = self.function.as_ref().unwrap();
        let mut sofiles: Vec<String> = Vec::from(self.files.as_ref().unwrap().as_slice());

        // Load dependencies from first file
        if self.is_ref {
            let f_data: Vec<u8> = fs::read(sofiles[0].as_str())
                .with_context(|| format!("Could not read file \"{}\"", sofiles[0]))?;
            let elf_ref = elf::Elf::parse(f_data.as_slice())
                .with_context(|| "Could not parse reference as ELF")?;
            sofiles.extend(
                elf_ref
                    .libraries
                    .iter()
                    .map(|l| String::from_str(l).unwrap()),
            );
        };

        /* if ld paths were specified, try to resolve file names */
        if let Some(paths) = &self.paths {
            for so in sofiles.iter_mut() {
                let so_path = PathBuf::from_str(so).unwrap();
                if so_path.is_relative() {
                    for p in paths.iter() {
                        let full_path = p.join(&so_path);
                        if full_path.is_file() {
                            *so = String::from_str(full_path.to_str().unwrap()).unwrap();
                        }
                    }
                }
            }
        }
        for f in sofiles.iter() {
            let f_data = fs::read(f).with_context(|| format!("Could not read file {}", f))?;
            let elf_file = elf::Elf::parse(f_data.as_slice())
                .with_context(|| format!("Could not parse {} as ELF", f))?;

            let strtab = elf_file.dynstrtab;

            let found = elf_file
                .dynsyms
                .iter()
                .any(|s| !s.is_import() && strtab.get_at(s.st_name) == Some(fun));
            if found {
                println!("{}", f);
            }
        }
        /* Return empty Vec as we output directly on stdout */
        Ok(Vec::<u8>::new())
    }
}
