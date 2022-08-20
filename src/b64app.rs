use crate::applet::Applet;
use crate::applet::SliceExt;
use clap::{arg, App, SubCommand};
use std::process;

pub struct B64EncApplet {
    encoding: base64::Config,
}

impl Applet for B64EncApplet {
    fn command(&self) -> &'static str {
        "b64"
    }
    fn description(&self) -> &'static str {
        "base64 encode"
    }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command())
            .about(self.description())
            .arg(arg!(-u --URL "Use URL-safe base64"))
            .arg(arg!([value] "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            encoding: base64::STANDARD,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self {
            encoding: if args.is_present("URL") {
                base64::URL_SAFE
            } else {
                base64::STANDARD
            },
        })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        base64::encode_config(&val, self.encoding)
            .as_bytes()
            .to_vec()
    }
}

pub struct B64DecApplet {
    encoding: base64::Config,
    strict: bool,
}

impl Applet for B64DecApplet {
    fn command(&self) -> &'static str {
        "d64"
    }
    fn description(&self) -> &'static str {
        "base64 decode"
    }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command())
            .about(self.description())
            .arg(arg!(-u --URL "Use URL-safe base64"))
            .arg(arg!(-s --strict "strict decoding, error on invalid data"))
            .arg(arg!([value] "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            encoding: base64::STANDARD.decode_allow_trailing_bits(true),
            strict: false,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self {
            encoding: if args.is_present("URL") {
                base64::URL_SAFE.decode_allow_trailing_bits(true)
            } else {
                self.encoding
            },
            strict: args.is_present("strict"),
        })
    }

    /* b64_decode. With two modes:
     * - strict: decode until the end of the valid base64
     * - lenient: decode the b64 input until the first invalid byte
     *   and return the decoded data concatenated with the rest
     */
    fn process(&self, b64val: Vec<u8>) -> Vec<u8> {
        let mut trimmed: Vec<u8> = b64val.trim().into();

        // If the length is invalid, decode up to the supplementary bytes
        if trimmed.len() % 4 != 0 && !self.strict {
            let end = trimmed.len() - (trimmed.len() % 4);
            let mut decoded = self.process((&trimmed[0..end]).to_vec());
            decoded.extend_from_slice(&trimmed[end..]);
            return decoded;
        }

        let decoded = base64::decode_config(&trimmed, self.encoding);
        match decoded {
            Ok(res) => res,
            Err(e) => {
                if self.strict {
                    eprintln!("Decoding base64 failed: {}", e);
                    process::exit(1);
                } else {
                    match e {
                        base64::DecodeError::InvalidLastSymbol(offset, _)
                        | base64::DecodeError::InvalidByte(offset, _) => {
                            let mut end = trimmed.split_off(offset);
                            let mut decoded = self.process(trimmed);
                            if !self.strict {
                                decoded.append(&mut end);
                            }
                            decoded
                        }
                        // Should not happen since we handle trailing data
                        // before in non-strict mode
                        base64::DecodeError::InvalidLength => {
                            panic!("Decoding base64 failed: {}", e)
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b64_inv_lenient() {
        let d64 = B64DecApplet {
            strict: false,
            encoding: base64::STANDARD,
        };
        assert_eq!(
            "::::".as_bytes().to_vec(),
            d64.process("::::".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_b64_enc() {
        let b64 = B64EncApplet {
            encoding: base64::STANDARD,
        };
        // https://tools.ietf.org/html/rfc4648#page-12
        assert_eq!("".as_bytes().to_vec(), b64.process("".as_bytes().to_vec()));
        assert_eq!(
            "Zg==".as_bytes().to_vec(),
            b64.process("f".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm8=".as_bytes().to_vec(),
            b64.process("fo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9v".as_bytes().to_vec(),
            b64.process("foo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYg==".as_bytes().to_vec(),
            b64.process("foob".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmE=".as_bytes().to_vec(),
            b64.process("fooba".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmFy".as_bytes().to_vec(),
            b64.process("foobar".as_bytes().to_vec())
        );

        let test = [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e].to_vec();
        assert_eq!("FPucA9l+".as_bytes().to_vec(), b64.process(test));
    }

    #[test]
    fn test_b64_url_enc() {
        let b64 = B64EncApplet {
            encoding: base64::URL_SAFE.decode_allow_trailing_bits(true),
        };
        // https://tools.ietf.org/html/rfc4648#page-12
        assert_eq!("".as_bytes().to_vec(), b64.process("".as_bytes().to_vec()));
        assert_eq!(
            "Zg==".as_bytes().to_vec(),
            b64.process("f".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm8=".as_bytes().to_vec(),
            b64.process("fo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9v".as_bytes().to_vec(),
            b64.process("foo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYg==".as_bytes().to_vec(),
            b64.process("foob".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmE=".as_bytes().to_vec(),
            b64.process("fooba".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmFy".as_bytes().to_vec(),
            b64.process("foobar".as_bytes().to_vec())
        );
        assert_eq!(
            "ZZ-A".as_bytes().to_vec(),
            b64.process([0x65, 0x9F, 0x80].to_vec())
        );

        let test = [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e].to_vec();
        assert_eq!("FPucA9l-".as_bytes().to_vec(), b64.process(test));
    }

    #[test]
    fn test_encode_and_back() {
        let d64 = B64DecApplet {
            strict: true,
            encoding: base64::STANDARD,
        };
        let b64 = B64EncApplet {
            encoding: base64::STANDARD,
        };

        let to_enc = [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf, 0x20].to_vec();
        assert_eq!(to_enc, d64.process(b64.process(to_enc.clone())));
    }

    #[test]
    fn test_encode_and_back_url() {
        let d64 = B64DecApplet {
            strict: true,
            encoding: base64::URL_SAFE,
        };
        let b64 = B64EncApplet {
            encoding: base64::URL_SAFE,
        };

        let to_enc = [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf, 0x20].to_vec();
        assert_eq!(to_enc, d64.process(b64.process(to_enc.clone())));
    }
}
