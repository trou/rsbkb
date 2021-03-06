use crate::applet::Applet;
use crate::applet::SliceExt;
use clap::{App, SubCommand};

pub struct HexApplet { }

impl Applet for HexApplet {
    fn command(&self) -> &'static str { "hex" }
    fn description(&self) -> &'static str { "hex encode" }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(HexApplet { })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        hex::encode(&val).as_bytes().to_vec()
    }

    fn new() -> Box<dyn Applet> {
        Box::new(HexApplet {})
    }
}

pub struct UnHexApplet { hexonly : bool, strict: bool }

impl UnHexApplet {
    fn hex_decode_hexonly(&self, val: Vec<u8>) -> Vec<u8> {
        let mut trimmed : Vec<u8> = val.trim().into();
        let res = hex::decode(&trimmed);
        if self.strict {
            return res.expect("Decoding hex failed");
        }
        /* remove spaces */
        trimmed.retain(|&x| x != 0x20);
        let res = hex::decode(&trimmed);
        match res {
            Ok(decoded) => return decoded,
            Err(e) => match e {
                hex::FromHexError::InvalidHexCharacter {c: _, index} => {
                    let mut end = trimmed.split_off(index);
                    let mut decoded = self.hex_decode_hexonly(trimmed);
                    decoded.append(&mut end);
                    return decoded;
                },
                hex::FromHexError::OddLength => {
                    // TODO: refactor
                    let mut end = trimmed.split_off(trimmed.len()-1);
                    let mut decoded = self.hex_decode_hexonly(trimmed);
                    decoded.append(&mut end);
                    return decoded;
                },
                _ => panic!("{}", e)
            }
        }
    }

    fn hex_decode_all(&self, hexval: Vec<u8>) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        let iter = &mut hexval.windows(2);
        let mut last : &[u8] = &[];
        loop {
            let chro = iter.next();
            let chr = match chro {
                None => { res.extend_from_slice(last) ; return res },
                Some(a) => a
            };

            if (chr[0] as char).is_digit(16) && (chr[1] as char).is_digit(16) {
                res.append(&mut hex::decode(chr).expect("hex decoding failed"));
                /* make sure we dont miss the last char if we have something like
                 * "41 " as input */
                let next_win = iter.next().unwrap_or(&[]);
                if next_win.len() > 1 {
                    last = &next_win[1..2]
                } else {
                    last = &[]
                };
            } else {
                res.extend_from_slice(&chr[0..1]);
                last = &chr[1..2];
            }
        }
    }
}

impl Applet for UnHexApplet {
    fn command(&self) -> &'static str { "unhex" }
    fn description(&self) -> &'static str { "Decode hex data" }

    fn new() -> Box<dyn Applet> {
        Box::new(UnHexApplet {hexonly: false, strict : false})
    }

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg_from_usage("-o --hex-only 'expect only hex data, stop at first non-hex byte (but copy the rest)'")
             .arg_from_usage("-s --strict 'strict decoding, error on invalid data'")
             .arg_from_usage("[value] 'input value, reads from stdin in not present'")
             .after_help("By default, decode all hex data in the input, regardless of garbage in-between.")
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(UnHexApplet { hexonly: args.is_present("hex-only"), strict : args.is_present("strict")})
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        if self.hexonly {
            self.hex_decode_hexonly(val)
        } else {
            self.hex_decode_all(val)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        let hex = HexApplet {};
        assert_eq!(String::from_utf8(hex.process([0, 0xFF].to_vec())).unwrap(), "00ff");
    }

    #[test]
    fn test_unhex_hexonly() {
        let unhex = UnHexApplet {strict: false, hexonly: true};
        assert_eq!(unhex.process("01 23 45 67 89 ab cd ef".as_bytes().to_vec()), [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        assert_eq!(unhex.process("0123456789abcdef".as_bytes().to_vec()), [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
    }

    #[test]
    fn test_unhex() {
        let unhex = UnHexApplet {strict: false, hexonly: false};
        assert_eq!(unhex.process("test52af ".as_bytes().to_vec()), [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf, 0x20]);
        assert_eq!(unhex.process("test52af".as_bytes().to_vec()), [0x74, 0x65, 0x73, 0x74, 0x52, 0xaf]);
        assert_eq!(unhex.process("!52af".as_bytes().to_vec()), [0x21, 0x52, 0xaf]);
        assert_eq!(unhex.process("!5 2af".as_bytes().to_vec()), [0x21, 0x35, 0x20, 0x2a, 0x66]);
    }

    #[test]
    #[should_panic(expected = "Decoding hex failed: OddLength")]
    fn test_unhex_strict_hexonly() {
        let unhex = UnHexApplet {strict: true, hexonly: true};
        unhex.process("01 23 45 67 89 ab cd ef".as_bytes().to_vec());
    }

    #[test]
    #[should_panic(expected = "Decoding hex failed: InvalidHexCharacter")]
    fn test_unhex_invalid() {
        let unhex = UnHexApplet {strict: true, hexonly: true};
        unhex.process("01at".as_bytes().to_vec());
    }
}
