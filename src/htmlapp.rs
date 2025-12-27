use std::borrow::Cow;

use crate::applet::Applet;
use crate::urlapp::build_custom_table;
use anyhow::Result;
use clap::{arg, Command};
use htmlentity::entity::{
    decode, encode, encode_char, encode_with, CharacterSet, EncodeType, EntityType,
};
pub struct HtmlEntApplet {
    table: Option<[bool; 256]>,
    decode: bool,
}

impl Applet for HtmlEntApplet {
    fn command(&self) -> &'static str {
        "htmlent"
    }
    fn description(&self) -> &'static str {
        "encode/decode HTML entities"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            table: None,
            decode: false,
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(
                arg!(-c --"custom" <custom> "string specifying chars to encode")
                    .conflicts_with("decode"),
            )
            .arg(arg!(-d --decode  "decode entities"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
            .after_help("By default, encode named and non-ASCII chars.")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        if args.contains_id("custom") {
            let mut table = [false; 256];
            let custom = args.get_one::<String>("custom").unwrap();
            build_custom_table("", custom, &mut table);
            return Ok(Box::new(Self {
                table: Some(table),
                decode: args.get_flag("decode"),
            }));
        };

        Ok(Box::new(Self {
            table: None,
            decode: args.get_flag("decode"),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        if self.decode {
            Ok(decode(&val[..]).into_bytes())
        } else {
            if let Some(table) = self.table {
                let enc_with_table = {
                    |c: &char, et: &EncodeType| {
                        if table[*c as u8 as usize] {
                            let enc_char_opt = encode_char(c, et);
                            if let Some(enc_char) = enc_char_opt {
                                (true, Some((EntityType::Named, Cow::from(enc_char.data()))))
                            } else {
                                (false, None)
                            }
                        } else {
                            (false, None)
                        }
                    }
                };
                Ok(encode_with(&val, &EncodeType::NamedOrHex, enc_with_table).into_bytes())
            } else {
                Ok(encode(
                    &val[..],
                    &EncodeType::NamedOrHex,
                    &CharacterSet::SpecialCharsAndNonASCII,
                )
                .into_bytes())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_htmlent_cli_arg_encode() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["htmlent", "aA<>&"])
            .assert()
            .stdout("aA&lt;&gt;&amp;")
            .success();
    }

    #[test]
    fn test_htmlent_cli_arg_decode() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["htmlent", "-d", "&lt;&gt;&amp;"])
            .assert()
            .stdout("<>&")
            .success();
    }

    #[test]
    fn test_htmlent_cli_arg_custom() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["htmlent", "-c", "aA", "aA<>&"])
            .assert()
            .stdout("&61;&41;<>&")
            .success();
    }

    #[test]
    fn test_htmlent_stdin() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["htmlent"])
            .write_stdin("aA<>&")
            .assert()
            .stdout("aA&lt;&gt;&amp;")
            .success();
    }

    #[test]
    fn test_htmlent_encode() {
        let htmlent = HtmlEntApplet {
            table: None,
            decode: false,
        };
        let encoded = htmlent
            .process("aA<>&".as_bytes().to_vec())
            .expect("encoding failed");
        assert_eq!(String::from_utf8(encoded).unwrap(), "aA&lt;&gt;&amp;");
    }

    #[test]
    fn test_htmlent_decode() {
        let htmlent = HtmlEntApplet {
            table: None,
            decode: true,
        };
        let decoded = htmlent
            .process("&lt;&gt;&amp;".as_bytes().to_vec())
            .expect("decoding failed");
        assert_eq!(String::from_utf8(decoded).unwrap(), "<>&");
    }

    #[test]
    fn test_htmlent_encode_custom() {
        let mut table = [false; 256];
        build_custom_table("", "aA", &mut table);
        let htmlent = HtmlEntApplet {
            table: Some(table),
            decode: false,
        };
        let encoded = htmlent
            .process("aA<>&".as_bytes().to_vec())
            .expect("encoding failed");
        assert_eq!(String::from_utf8(encoded).unwrap(), "&61;&41;<>&");
    }

    #[test]
    fn test_htmlent_encdec() {
        let htmlent_encode = HtmlEntApplet {
            table: None,
            decode: false,
        };
        let htmlent_decode = HtmlEntApplet {
            table: None,
            decode: true,
        };
        let test_string = "aA<>&\"'";
        let encoded = htmlent_encode
            .process(test_string.as_bytes().to_vec())
            .expect("encoding failed");
        let decoded = htmlent_decode.process(encoded).expect("decoding failed");
        assert_eq!(String::from_utf8(decoded).unwrap(), test_string);
    }
}
