use crate::applet::Applet;
use crate::applet::SliceExt;
use anyhow::Result;
use clap::{arg, Command};

pub struct UrlEncApplet {
    excluded: String,
}

impl Applet for UrlEncApplet {
    fn command(&self) -> &'static str {
        "urlenc"
    }
    fn description(&self) -> &'static str {
        "URL encode"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            excluded: "".to_string(),
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-e --"exclude-chars" <chars>  "a string of chars to exclude from encoding"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
            .after_help("By default, encode all non alphanumeric characters in the input.")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        if args.contains_id("exclude-chars") {
            let chars: &String = args.get_one::<String>("exclude-chars").unwrap();

            Ok(Box::new(Self {
                excluded: chars.to_string(),
            }))
        } else {
            Ok(Box::new(Self {
                excluded: "".to_string(),
            }))
        }
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let mut table = [false; 256];

        for i in 0..255 {
            let c = char::from_u32(i).unwrap();
            if !c.is_ascii_graphic() {
                table[i as usize] = true;
            } else {
                if matches!(
                    c,
                    '!' | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '\''
                        | '('
                        | ')'
                        | '*'
                        | '+'
                        | ','
                        | '/'
                        | ':'
                        | ';'
                        | '='
                        | '?'
                        | '@'
                        | '['
                        | ']'
                ) && !self.excluded.contains(c)
                {
                    table[i as usize] = true;
                }
            }
        }
        let mut encoded = Vec::with_capacity(val.len());
        for b in val.iter() {
            if table[*b as usize] {
                encoded.extend_from_slice(format!("%{:02x}", *b).as_bytes());
            } else {
                encoded.push(*b);
            };
        }
        Ok(encoded)
    }
}

pub struct UrlDecApplet {}

impl Applet for UrlDecApplet {
    fn command(&self) -> &'static str {
        "urldec"
    }
    fn description(&self) -> &'static str {
        "URL decode"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, urlval: Vec<u8>) -> Result<Vec<u8>> {
        let trimmed: Vec<u8> = urlval.trim().into();
        let decoded: Vec<u8> = percent_encoding::percent_decode(&trimmed).collect();
        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlenc_cli_arg() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["urlenc", "aAé!,"])
            .assert()
            .stdout("aA%c3%a9%21%2c")
            .success();
    }

    #[test]
    fn test_urlenc_cli_arg_exclude() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["urlenc", "-e", "!,", "aAé!,"])
            .assert()
            .stdout("aA%c3%a9!,")
            .success();
    }

    #[test]
    fn test_urlenc_stdin() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["urlenc"])
            .write_stdin("aAé!,")
            .assert()
            .stdout("aA%c3%a9%21%2c")
            .success();
    }

    #[test]
    fn test_urlenc() {
        let urlenc = UrlEncApplet {
            excluded: "".to_string(),
        };
        let encoded = urlenc
            .process("aA!,é".as_bytes().to_vec())
            .expect("encoding failed");
        assert_eq!(String::from_utf8(encoded).unwrap(), "aA%21%2c%c3%a9");
    }

    #[test]
    fn test_urlencdec() {
        let urlenc = UrlEncApplet {
            excluded: "".to_string(),
        };
        let urldec = UrlDecApplet {};
        let test_string = "aA!,é";
        let encoded = urlenc
            .process(test_string.as_bytes().to_vec())
            .expect("encoding failed");
        let decoded = urldec.process(encoded).expect("decoding failed");
        assert_eq!(String::from_utf8(decoded).unwrap(), test_string);
    }
}
