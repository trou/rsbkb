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
    files: Option<Vec<PathBuf>>,
    // First .so is a binary to look for dependencies in
    is_ref: bool,
    // LD_LIBRARY_PATH equivalent
    paths: Option<Vec<PathBuf>>,
    // don't show warnings
    quiet: bool,
    // skip symbolic links in results
    skip_symlinks: bool
}

fn parse_ld_so_conf(ldconf_path: &str) -> Result<Vec<PathBuf>> {
    let conf_file = fs::read_to_string(ldconf_path)
        .with_context(|| format!("Could not read config file \"{}\"", ldconf_path))?;
    let conf_lines = conf_file.split('\n');

    // Handle "normal" lines: skip comments and includes
    let mut ldpaths: Vec<PathBuf> = conf_lines
        .clone()
        .filter(|p| !p.starts_with("include ") && p.get(0..1).unwrap_or("#") != "#") // Skip empty lines and comments
        .map(|p| PathBuf::from_str(p).unwrap())
        .collect::<Vec<PathBuf>>();

    // Handle includes: get a list of included paths
    let includes = conf_lines
        .filter(|l| l.starts_with("include "))
        .map(|p| p.replace("include ", ""));

    for inc_path in includes.into_iter() {
        for inc_match in glob::glob(inc_path.as_str()).unwrap().flatten() {
            ldpaths.extend(parse_ld_so_conf(inc_match.to_str().unwrap())?);
        }
    }
    Ok(ldpaths)
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
            .arg(arg!(-a --all "search in all '*.so*' files found in LDPATH").conflicts_with("ref"))
            .arg(arg!(-s --"skip-symlinks" "ignore symbolic links"))
            .arg(arg!(-r --ref  "use first file as reference ELF to get .so list from"))
            .arg(arg!(-q --quiet  "don't show warnings on invalid files"))
            .arg(arg!(-p --ldpath <LDPATH> "'\':\' separated list of paths to look for .so in'"))
            .arg(arg!(-l --ldconf [CONF]  "use config file to get LD paths, /etc/ld.so.conf is used if not specified"))
            .arg(arg!(<function>  "function to search"))
            .arg(
                arg!([files]...  "files to search in, optional if --all is set")
                    .required_unless_present("all"),
            )
            .after_long_help("Examples:\n
                            'findso -a memcpy -l': search for 'memcpy' in all .so files in paths defined in /etc/ld.so.conf\n
                            'findso -r memcpy /bin/ls -l': search for memcpy in all .so files referenced in /bin/ls in system paths\n
                            'findso -q memcpy /usr/lib32/*.so*': search for memcpy in all given files
                            'findso -p /usr/lib32/:/usr/lib64/ -a -q memcpy': search for memcpy in given paths")
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
            quiet: false,
            skip_symlinks: false,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let mut filenames: Vec<PathBuf> = Vec::new();
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
                let ld_so_conf = "/etc/ld.so.conf".to_string();
                let ldconf = args.get_one::<String>("ldconf").or(Some(&ld_so_conf));
                let ldpaths = parse_ld_so_conf(ldconf.unwrap())?;
                paths.extend(ldpaths);
            }

            // check that paths are actually valid directories
            paths.retain(|p| p.is_dir());

            Some(paths)
        } else {
            None
        };

        if args.get_flag("all") {
            if let Some(paths_v) = &paths {
                for p in paths_v {
                    let so_files: Vec<PathBuf> = glob::glob(p.join("*.so*").to_str().unwrap())
                        .with_context(|| format!("Could not find .so files in {}", p.display()))?
                        .map(|p| p.expect("could not find so"))
                        .collect();
                    filenames.extend(so_files);
                }
            } else {
                anyhow::bail!("--all without any paths");
            }
        } else {
            filenames.extend(
                args.get_many::<String>("files")
                    .unwrap()
                    .map(|x| PathBuf::from_str(x).unwrap()),
            );
        }

        Ok(Box::new(Self {
            files: Some(filenames),
            function: Some(function_val.to_string()),
            is_ref: args.get_flag("ref"),
            paths,
            quiet: args.get_flag("quiet"),
            skip_symlinks: args.get_flag("skip-symlinks"),
        }))
    }

    fn process(&self, _val: Vec<u8>) -> Result<Vec<u8>> {
        let fun = self.function.as_ref().unwrap();
        let mut sofiles: Vec<PathBuf> = Vec::from(self.files.as_ref().unwrap().as_slice());

        // Load dependencies from first file
        if self.is_ref {
            let f_data: Vec<u8> = fs::read(&sofiles[0])
                .with_context(|| format!("Could not read file \"{}\"", sofiles[0].display()))?;
            let elf_ref = elf::Elf::parse(f_data.as_slice())
                .with_context(|| "Could not parse reference as ELF")?;
            sofiles.extend(
                elf_ref
                    .libraries
                    .iter()
                    .map(|l| PathBuf::from_str(l).unwrap()),
            );
        };

        let mut resolved_sofiles: Vec<PathBuf> = Vec::new();

        /* if ld paths were specified, try to resolve file names */
        if let Some(paths) = &self.paths {
            for so_path in sofiles.iter_mut() {
                if so_path.is_relative() {
                    for p in paths.iter() {
                        let full_path = p.join(&so_path);
                        if full_path.is_file() {
                            if self.skip_symlinks && !full_path.is_symlink() {
                                resolved_sofiles.push(full_path);
                            }
                        }
                    }
                }
            }
        } else {
            resolved_sofiles.extend(sofiles);
        }

        for f in resolved_sofiles.iter() {
            // Skip directories
            if fs::metadata(f)
                .with_context(|| format!("Could not open {}", f.display()))?
                .is_dir()
            {
                continue;
            }
            let f_data =
                fs::read(f).with_context(|| format!("Could not read file {}", f.display()))?;
            let elf_file = elf::Elf::parse(f_data.as_slice());
            if let Ok(elf_file) = elf_file {
                let strtab = elf_file.dynstrtab;

                let found = elf_file
                    .dynsyms
                    .iter()
                    .any(|s| !s.is_import() && strtab.get_at(s.st_name) == Some(fun));
                if found && !(self.skip_symlinks && f.is_symlink()) {
                    println!("{}", f.display());
                }
            } else if !self.quiet {
                eprintln!("Could not parse {} as ELF", f.display());
            }
        }

        /* Return empty Vec as we output directly on stdout */
        Ok(Vec::<u8>::new())
    }
}
