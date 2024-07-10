use crate::applet::Applet;
use crate::applet::SliceExt;
use anyhow::{Context, Result};
use base64::engine::general_purpose;
use base64::engine::Engine;
use clap::{arg, Command};

pub struct B64EncApplet {
    engine: general_purpose::GeneralPurpose,
}

impl Applet for B64EncApplet {
    fn command(&self) -> &'static str {
        "b64"
    }
    fn description(&self) -> &'static str {
        "base64 encode"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-u --URL "Use URL-safe base64"))
            .arg(
                arg!(-a --alphabet <ALPHABET> "specify custom alphabet")
                    .conflicts_with("URL")
                    .required(false),
            )
            .arg(arg!([value] "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            engine: general_purpose::STANDARD,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            engine: if args.get_flag("URL") {
                general_purpose::URL_SAFE
            } else if args.contains_id("alphabet") {
                let custom = base64::alphabet::Alphabet::new(
                    args.get_one::<String>("alphabet")
                        .with_context(|| "alphabet is not specified")?,
                )
                .with_context(|| "Invalid alphabet")?;
                base64::engine::GeneralPurpose::new(&custom, base64::engine::general_purpose::PAD)
            } else {
                general_purpose::STANDARD
            },
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.engine.encode(val).as_bytes().to_vec())
    }
}

pub struct B64DecApplet {
    engine: general_purpose::GeneralPurpose,
    strict: bool,
}

impl Applet for B64DecApplet {
    fn command(&self) -> &'static str {
        "d64"
    }

    fn description(&self) -> &'static str {
        "base64 decode"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-u --URL "use URL-safe base64"))
            .arg(
                arg!(-a --alphabet <ALPHABET> "specify custom alphabet")
                    .conflicts_with("URL")
                    .required(false),
            )
            .arg(arg!(-s --strict "strict decoding, error on invalid data"))
            .arg(arg!([value] "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        let engine_cfg =
            base64::engine::GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true);
        Box::new(Self {
            engine: general_purpose::GeneralPurpose::new(&base64::alphabet::STANDARD, engine_cfg),
            strict: false,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let engine_cfg =
            base64::engine::GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true);
        let alphabet = if args.get_flag("URL") {
            base64::alphabet::URL_SAFE
        } else if args.contains_id("alphabet") {
            base64::alphabet::Alphabet::new(
                args.get_one::<String>("alphabet")
                    .with_context(|| "alphabet is not specified")?,
            )
            .with_context(|| "Invalid alphabet")?
        } else {
            base64::alphabet::STANDARD
        };
        Ok(Box::new(Self {
            engine: general_purpose::GeneralPurpose::new(&alphabet, engine_cfg),
            strict: false,
        }))
    }

    /* b64_decode. With two modes:
     * - strict: decode until the end of the valid base64
     * - lenient: decode the b64 input until the first invalid byte
     *   and return the decoded data concatenated with the rest
     */
    fn process(&self, b64val: Vec<u8>) -> Result<Vec<u8>> {
        let mut trimmed: Vec<u8> = b64val.trim().into();

        // If the length is invalid, decode up to the supplementary bytes
        if trimmed.len() % 4 != 0 && !self.strict {
            let end = trimmed.len() - (trimmed.len() % 4);
            let mut decoded = self.process(trimmed[0..end].to_vec())?;
            decoded.extend_from_slice(&trimmed[end..]);
            return Ok(decoded);
        }

        let decoded = self.engine.decode(b64val);
        match decoded {
            Ok(res) => Ok(res),
            Err(ref e) => {
                if self.strict {
                    decoded.with_context(|| "Decoding base64 failed")
                } else {
                    match e {
                        base64::DecodeError::InvalidLastSymbol(offset, _)
                        | base64::DecodeError::InvalidByte(offset, _) => {
                            let mut end = trimmed.split_off(*offset);
                            let mut decoded = self.process(trimmed)?;
                            if !self.strict {
                                decoded.append(&mut end);
                            }
                            Ok(decoded)
                        }
                        // Should not happen since we handle trailing data
                        // before in non-strict mode
                        base64::DecodeError::InvalidLength(_) => {
                            decoded.with_context(|| "Decoding base64 failed")
                        }
                        base64::DecodeError::InvalidPadding => {
                            decoded.with_context(|| "Decoding base64 failed: invalid padding")
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
        let engine_cfg =
            base64::engine::GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true);
        let engine = general_purpose::GeneralPurpose::new(&base64::alphabet::STANDARD, engine_cfg);

        let d64 = B64DecApplet {
            strict: false,
            engine: engine,
        };
        assert_eq!(
            "::::".as_bytes().to_vec(),
            d64.process_test("::::".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_b64_enc() {
        let b64 = B64EncApplet {
            engine: general_purpose::STANDARD,
        };
        // https://tools.ietf.org/html/rfc4648#page-12
        assert_eq!(
            "".as_bytes().to_vec(),
            b64.process_test("".as_bytes().to_vec())
        );
        assert_eq!(
            "Zg==".as_bytes().to_vec(),
            b64.process_test("f".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm8=".as_bytes().to_vec(),
            b64.process_test("fo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9v".as_bytes().to_vec(),
            b64.process_test("foo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYg==".as_bytes().to_vec(),
            b64.process_test("foob".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmE=".as_bytes().to_vec(),
            b64.process_test("fooba".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmFy".as_bytes().to_vec(),
            b64.process_test("foobar".as_bytes().to_vec())
        );

        let test = [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e].to_vec();
        assert_eq!("FPucA9l+".as_bytes().to_vec(), b64.process_test(test));
    }

    #[test]
    fn test_b64_url_enc() {
        let b64 = B64EncApplet {
            engine: general_purpose::URL_SAFE,
        };
        // https://tools.ietf.org/html/rfc4648#page-12
        assert_eq!(
            "".as_bytes().to_vec(),
            b64.process_test("".as_bytes().to_vec())
        );
        assert_eq!(
            "Zg==".as_bytes().to_vec(),
            b64.process_test("f".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm8=".as_bytes().to_vec(),
            b64.process_test("fo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9v".as_bytes().to_vec(),
            b64.process_test("foo".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYg==".as_bytes().to_vec(),
            b64.process_test("foob".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmE=".as_bytes().to_vec(),
            b64.process_test("fooba".as_bytes().to_vec())
        );
        assert_eq!(
            "Zm9vYmFy".as_bytes().to_vec(),
            b64.process_test("foobar".as_bytes().to_vec())
        );
        assert_eq!(
            "ZZ-A".as_bytes().to_vec(),
            b64.process_test([0x65, 0x9F, 0x80].to_vec())
        );

        let test = [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e].to_vec();
        assert_eq!("FPucA9l-".as_bytes().to_vec(), b64.process_test(test));
    }

    #[test]
    fn test_encode_and_back() {
        let b64 = B64EncApplet {
            engine: general_purpose::STANDARD,
        };
        let d64 = B64DecApplet {
            strict: true,
            engine: general_purpose::STANDARD,
        };

        let to_enc = [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf, 0x20].to_vec();
        assert_eq!(to_enc, d64.process_test(b64.process_test(to_enc.clone())));
    }

    #[test]
    fn test_encode_and_back_url() {
        let b64 = B64EncApplet {
            engine: general_purpose::URL_SAFE,
        };
        let d64 = B64DecApplet {
            strict: true,
            engine: general_purpose::URL_SAFE,
        };

        let to_enc = [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf, 0x20].to_vec();
        assert_eq!(to_enc, d64.process_test(b64.process_test(to_enc.clone())));
    }
}
