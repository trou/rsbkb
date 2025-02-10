use crate::applet::Applet;
use crate::applet::SliceExt;
use anyhow::{Context, Result};
use clap::{arg, Command};
use htmlentity::entity::{decode, encode, CharacterSet, EncodeType};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum EscType {
    #[default]
    Generic,
    Single,
    Shell,
    Bash,
    BashSingle,
    HTMLEntities,
}

const SHELL_CHARS: &[u8; 4] = b"`$\"\\";

// Note that bash is crazy regarding '!'
// echo "\!" will output \!
// ref: https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
// > If enabled, history expansion will be performed unless an ‘!’ appearing in double quotes is
// > escaped using a backslash. The backslash preceding the ‘!’ is not removed.
const BASH_CHARS: &[u8; 5] = b"`$\"\\!";

trait SliceEsc {
    fn escape(&self, esc_type: &EscType) -> Vec<u8>;
    fn escape_chars(&self, chars: &[u8]) -> Vec<u8>;
    fn escape_bash_single(&self) -> Vec<u8>;
}

impl SliceEsc for [u8] {
    fn escape(&self, esc_type: &EscType) -> Vec<u8> {
        match esc_type {
            EscType::Generic | EscType::Single => self.escape_ascii().collect(),
            EscType::Shell => self.escape_chars(SHELL_CHARS),
            EscType::Bash => self.escape_chars(BASH_CHARS),
            EscType::BashSingle => self.escape_bash_single(),
            EscType::HTMLEntities => encode(
                &self,
                &EncodeType::NamedOrHex,
                &CharacterSet::SpecialCharsAndNonASCII,
            )
            .into_bytes(),
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
            if parts.peek().is_some() {
                // https://stackoverflow.com/a/1250279
                res.extend_from_slice(b"'\"'\"'")
            }
        }
        res
    }
}

pub struct EscapeApplet {
    esc_type: EscType,
    no_quote: bool,
    no_detect: bool,
    multiline: bool,
}

impl Applet for EscapeApplet {
    fn command(&self) -> &'static str {
        "escape"
    }
    fn description(&self) -> &'static str {
        "backslash-escape input strings"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-m --multiline "expect multiline string, do not trim input"))
            .arg(arg!(-d --"no-detect" "do not detect surrounding quotes"))
            .arg(arg!(-n --"no-quote" "do not wrap output in quotes"))
            .arg(
                arg!(-t --type [type] "type of escape")
                    .value_parser(clap::builder::EnumValueParser::<EscType>::new())
                    .default_value("generic"),
            )
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            esc_type: args.get_one::<EscType>("type").unwrap().clone(),
            no_quote: args.get_flag("no-quote"),
            no_detect: args.get_flag("no-detect"),
            multiline: args.get_flag("multiline"),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let to_escape = if self.multiline {
            val
        } else {
            val.trim().into()
        };

        // Detect (unless no_detect) surrounding quotes to:
        //  - remove them
        //  - escape
        //  - restore them
        let (quote, to_escape_nq) = if !self.no_detect {
            // set unwrap_or result to other chars, just for
            // simpler code
            let first = *to_escape.first().unwrap_or(&b' ');
            let last = *to_escape.last().unwrap_or(&b'*');

            // If quotes don't match or start/end chars are not quote,
            // return quote char according to escape type
            if first != last || (first != b'\'' && first != b'"') {
                match self.esc_type {
                    EscType::BashSingle | EscType::Single => (Some(b'\''), to_escape),
                    _ => (Some(b'"'), to_escape),
                }
            } else {
                // if we have matching quotes, return quote char and remove them
                let end_pos = to_escape.len() - 1;
                match first {
                    b'\'' => (Some(b'\''), to_escape[1..end_pos].to_vec()),
                    b'"' => (Some(b'"'), to_escape[1..end_pos].to_vec()),
                    _ => (None, to_escape),
                }
            }
        } else {
            // no_detect
            match self.esc_type {
                EscType::BashSingle | EscType::Single => (Some(b'\''), to_escape),
                _ => (Some(b'"'), to_escape),
            }
        };

        let escaped = to_escape_nq.escape(&self.esc_type);
        if self.no_quote || quote.is_none() {
            Ok(escaped)
        } else {
            let mut res = Vec::<u8>::with_capacity(escaped.len() + 2);
            res.push(quote.unwrap());
            res.extend(escaped);
            res.push(quote.unwrap());
            Ok(res)
        }
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            esc_type: EscType::Generic,
            no_quote: false,
            no_detect: false,
            multiline: false,
        })
    }
}

pub struct UnEscapeApplet {
    multiline: bool,
    html_entities: bool,
}

impl Applet for UnEscapeApplet {
    fn command(&self) -> &'static str {
        "unescape"
    }
    fn description(&self) -> &'static str {
        "(backslash) unescape input strings"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            multiline: false,
            html_entities: false,
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-t --html "unescape HTML entities"))
            .arg(arg!(-m --multiline "expect multiline string, do not trim input"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            multiline: args.get_flag("multiline"),
            html_entities: args.get_flag("html"),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        enum EscapeState {
            Backslash,
            Hex1,
            Hex2,
            Normal,
        }

        let to_unescape = if self.multiline {
            val
        } else {
            val.trim().into()
        };

        // Special case for HTMLEntities
        if self.html_entities {
            return Ok(decode(&to_unescape).into_bytes());
        };

        let mut res = Vec::with_capacity(to_unescape.len());
        let mut state = EscapeState::Normal;
        let mut hexchars: [u8; 2] = [0, 0];
        for c in to_unescape.iter() {
            state = match (state, c) {
                (EscapeState::Normal, b'\\') => EscapeState::Backslash,
                (EscapeState::Normal, c) => {
                    res.push(*c);
                    EscapeState::Normal
                }
                (EscapeState::Backslash, b'x') => EscapeState::Hex1,
                (EscapeState::Backslash, b't') => {
                    res.push(0x9);
                    EscapeState::Normal
                }
                (EscapeState::Backslash, b'n') => {
                    res.push(0xA);
                    EscapeState::Normal
                }
                (EscapeState::Backslash, b'r') => {
                    res.push(0xD);
                    EscapeState::Normal
                }
                (EscapeState::Backslash, c) => {
                    res.push(*c);
                    EscapeState::Normal
                }
                (EscapeState::Hex1, c) => {
                    hexchars[0] = *c;
                    EscapeState::Hex2
                }
                (EscapeState::Hex2, c) => {
                    hexchars[1] = *c;
                    res.push(
                        u8::from_str_radix(
                            std::str::from_utf8(&hexchars)
                                .context("invalid hex chars in escaped string")?,
                            16,
                        )
                        .context("invalid hex char in escaped string")?,
                    );
                    EscapeState::Normal
                }
            };
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_base_escape_arg_auto() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", r"te'st"])
            .assert()
            .stdout(r#""te\'st""#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_auto() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape"])
            .write_stdin("'te'st'\n") // by default, trim input so '\n' will be removed
            .assert()
            .stdout(r"'te\'st'")
            .success();
    }

    #[test]
    fn test_base_escape_stdin_no_detect() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-d"])
            // no detect mode will not try to determine enclosing quote type,
            // just escape them
            .write_stdin(r"'test'")
            .assert()
            .stdout(r#""\'test\'""#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_auto_multiline() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-m"])
            // multiline mode will not trim '\n', escaping them instead
            .write_stdin("te'st\nte\"st\n")
            .assert()
            .stdout(r#""te\'st\nte\"st\n""#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_bash_single() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-t", "bash-single"])
            .write_stdin("te'st")
            .assert()
            .stdout(r#"'te'"'"'st'"#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_bash() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-t", "bash"])
            .write_stdin(r#""!t"e`s$t""#)
            .assert()
            .stdout(r#""\!t\"e\`s\$t""#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_posix_shell() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-t", "shell"])
            .write_stdin(r#""!t"e`s$t""#)
            .assert()
            .stdout(r#""!t\"e\`s\$t""#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_single() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-t", "single"])
            .write_stdin(r#"sin'gle"#)
            .assert()
            .stdout(r#"'sin\'gle'"#)
            .success();
    }

    #[test]
    fn test_base_escape_stdin_single_noquote() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["escape", "-t", "single", "-n"])
            .write_stdin(r#"sin'gle"#)
            .assert()
            .stdout(r#"sin\'gle"#)
            .success();
    }
}
