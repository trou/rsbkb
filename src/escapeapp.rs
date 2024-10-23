use crate::applet::Applet;
use anyhow::{Result};
use clap::{arg, Command};
use crate::applet::SliceExt;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum EscType {
    #[default]
    Default,
    PosixShell,
    Bash,
    BashSingle,
}

const SHELL_CHARS: &[u8; 4] = b"`$\"\\";
const BASH_CHARS: &[u8; 5] = b"`$\"\\!";

trait SliceEsc {
    fn escape(&self, esc_type: &EscType) -> Vec<u8>;
    fn escape_chars(&self, chars: &[u8]) -> Vec<u8>;
    fn escape_bash_single(&self) -> Vec<u8>;
}

impl SliceEsc for [u8] {
    fn escape(&self, esc_type: &EscType) -> Vec<u8> {
        match esc_type {
            EscType::Default => self.escape_ascii().collect(),
            EscType::PosixShell => self.escape_chars(SHELL_CHARS),
            EscType::Bash => self.escape_chars(BASH_CHARS),
            EscType::BashSingle => self.escape_bash_single(),
        }
    }

    fn escape_chars(&self, chars: &[u8]) -> Vec<u8> {
        let mut res = Vec::<u8>::with_capacity(self.len());
        for c in self {
            if chars.contains(c) {
                res.push(b'\\');
            }
            res.push(*c);
        }
        res
    }

    fn escape_bash_single(&self) -> Vec<u8> {
        let mut res = Vec::<u8>::with_capacity(self.len());
        let mut parts = self.split(|b| *b == b'\'').peekable();
        while let Some(part) = parts.next() {
            res.extend_from_slice(part);
            if !parts.peek().is_none() {
                // https://stackoverflow.com/a/1250279
                res.extend_from_slice(b"'\"'\"'")
            }
        }
        res
    }
}

pub struct EscapeApplet {
    esc_type: EscType,
    no_quote: bool
}

impl Applet for EscapeApplet {
    fn command(&self) -> &'static str {
        "escape"
    }
    fn description(&self) -> &'static str {
        "escape input strings"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-n --"no-quote" "do not wrap output in quotes"))
            .arg(
                arg!(-t --type [type] "type of escape")
                    .value_parser(clap::builder::EnumValueParser::<EscType>::new())
                    .default_value("default"),
            )
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            esc_type: args.get_one::<EscType>("type").unwrap().clone(),
            no_quote: args.get_flag("no-quote")
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let escaped = val.trim().escape(&self.esc_type);
        if self.no_quote {
            return Ok(escaped)
        } else {
            let quote = match self.esc_type {
                EscType::BashSingle => b'\'',
                _ => b'"'
            };
            let mut res = Vec::<u8>::with_capacity(escaped.len()+2);
            res.push(quote);
            res.extend(escaped);
            res.push(quote);
            return Ok(res)
        }
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            esc_type: EscType::Default,
            no_quote: false
        })
    }
}

pub struct UnEscapeApplet {}

impl Applet for UnEscapeApplet {
    fn command(&self) -> &'static str {
        "unescape"
    }
    fn description(&self) -> &'static str {
        "unescape input strings"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        Ok(val)
    }
}
