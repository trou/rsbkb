use std::env;
use std::path::Path;
use std::io;
use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::{OpenOptions};
extern crate hex;
extern crate base64;
extern crate percent_encoding;
extern crate clap;
extern crate crc;
use clap::{Arg, App};
use crc::{crc16, crc32};

trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b' ' || *c == b'\n' || *c == b'\r'
        }

        fn is_not_whitespace(c: &u8) -> bool {
            !is_whitespace(c)
        }

        if let Some(first) = self.iter().position(is_not_whitespace) {
            if let Some(last) = self.iter().rposition(is_not_whitespace) {
                &self[first..last + 1]
            } else {
                unreachable!();
            }
        } else {
            &[]
        }
    }
}

/* b64_decode. With two modes:
 * - strict: decode until the end of the valid base64
 * - lenient: decode the b64 input until the first invalid byte
 *   and return the decoded data concatenated with the rest
 */
fn b64_decode(b64val: Vec<u8>, strict: bool) -> Vec<u8> {
    let mut trimmed : Vec<u8> = b64val.trim().into();

    // If the length is invalid, decode up to the supplementary bytes
    if trimmed.len()% 4 != 0 && !strict {
        let end = trimmed.len()-(trimmed.len() % 4);
        let mut decoded = b64_decode((&trimmed[0..end]).to_vec(), strict);
        decoded.extend_from_slice(&trimmed[end..]);
        return decoded;
    }

    let decoded = base64::decode_config(&trimmed, base64::STANDARD.decode_allow_trailing_bits(true));
    match decoded {
        Ok(res) => return res,
        Err(e) =>
                match e {
                    base64::DecodeError::InvalidLastSymbol(offset, _) |
                    base64::DecodeError::InvalidByte(offset, _) => {
                        let mut end = trimmed.split_off(offset);
                        let mut decoded = b64_decode(trimmed, strict);
                        if !strict {
                            decoded.append(&mut end);
                        }
                        return decoded;
                    },
                    _ =>  {println!("{}",e); panic!("Decoding base64 failed: {}", e) }}
    }
}

fn b64_encode(val: Vec<u8>) -> Vec<u8> {
    let encoded = base64::encode(&val)+"\n";
    return encoded.as_bytes().to_vec();
}

fn url_decode(urlval: Vec<u8>) -> Vec<u8> {
    let trimmed : Vec<u8> = urlval.trim().into();
    let decoded: Vec<u8> = percent_encoding::percent_decode(&trimmed).collect();
    return decoded;
}

fn url_encode(val: Vec<u8>) -> Vec<u8> {
    let encoded = percent_encoding::percent_encode(&val, percent_encoding::NON_ALPHANUMERIC).to_string();
    return (encoded+"\n").as_bytes().to_vec();
}

fn xor(xorkey: &str, val: Vec<u8>) -> Vec<u8> {
    let key_bytes = hex::decode(xorkey).expect("Xor key decoding failed");
    let inf_key = key_bytes.iter().cycle(); // Iterate endlessly over key bytes
    return val.iter().zip(inf_key).map (|(x, k)| x ^ k).collect();
}

fn hex_decode(hexval: Vec<u8>, strict: bool) -> Vec<u8> {
    let mut trimmed : Vec<u8> = hexval.trim().into();
    let res = hex::decode(&trimmed);
    if strict {
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
                let mut decoded = hex_decode(trimmed, strict);
                decoded.append(&mut end);
                return decoded;
            },
            hex::FromHexError::OddLength => {
                // TODO: refactor
                let mut end = trimmed.split_off(trimmed.len()-1);
                let mut decoded = hex_decode(trimmed, strict);
                decoded.append(&mut end);
                return decoded;
            },
            _ => panic!("{}", e)
        }
    }
}

/* Iterate over the whole "hexval", decoding only pairs of hex chars
 * and leaving the rest untouched */
fn hex_decode_all(hexval: Vec<u8>) -> Vec<u8> {
    let mut res: Vec<u8> = vec![];
    let ref mut iter = hexval.windows(2);
    loop {
        let chro = iter.next();
        let chr = match chro {
            None => return res,
            Some(a) => a
        };

        if (chr[0] as char).is_digit(16) && (chr[1] as char).is_digit(16) {
            res.append(&mut hex::decode(chr).expect("hex decoding failed"));
            iter.next();
        } else {
            res.extend_from_slice(&chr[0..1]);
        }
    }
}

enum Operation {
        HexDecode,
        HexDecodeAll,
        HexEncode,
        B64Decode,
        B64Encode,
        UrlDecode,
        UrlEncode,
        Xor,
        Crc16,
        Crc32,
        Slice,
}

fn num_from_str_safe(s: &str) -> Result<u64, std::num::ParseIntError> {
    if &s[0..2] == "0x" {
        return u64::from_str_radix(&s[2..], 16);
    } else {
        return s.parse();
    }
}

fn slice(args: clap::ArgMatches) -> Vec<u8> {
    let file = args.value_of("value").unwrap();
    let start_va = args.value_of("start").unwrap();
    let start: u64 = num_from_str_safe(start_va).expect("invalid start");

    let mut f = OpenOptions::new().read(true).write(true).open(file).expect("can't open file");
    f.seek(SeekFrom::Start(start)).expect("Seek failed");
    let mut res =  vec![];
    if args.is_present("end") {
        let end: u64 = num_from_str_safe(args.value_of("end").unwrap()).expect("Invalid end");
        if end < start {
            panic!("end < start");
        }
        let len: usize = (end-start) as usize;
        res.resize(len, 0);
        f.read_exact(&mut res).expect("Read failed");
    } else {
        f.read_to_end(&mut res).expect("Read failed");
    }
    return res.to_vec();
}

fn process(args: clap::ArgMatches , op: Operation, val: Vec<u8>) -> Vec<u8> {
    match op {
        Operation::HexDecode => return hex_decode(val, args.is_present("strict")),
        Operation::HexDecodeAll => return hex_decode_all(val),
        Operation::HexEncode => return (hex::encode(&val)+"\n").as_bytes().to_vec(),
        Operation::B64Decode => return b64_decode(val, args.is_present("strict")),
        Operation::B64Encode => return b64_encode(val),
        Operation::UrlDecode => return url_decode(val),
        Operation::UrlEncode => return url_encode(val),
        Operation::Xor => return xor(args.value_of("xorkey").unwrap(), val),
        Operation::Slice => return slice(args),
        Operation::Crc16 => return format!("{:08x}\n", crc16::checksum_x25(&val)).as_bytes().to_vec(),
        Operation::Crc32 => return format!("{:08x}\n", crc32::checksum_ieee(&val)).as_bytes().to_vec(),
    }
}



fn main() {
    let args: Vec<_>= env::args().collect();

    let arg0 = Path::new(&args[0]).file_name();
    let arg0 = match arg0 {
        Some(a) => a.to_str().unwrap().to_string(),
        None => panic!("No arg0"),
    };
    let mut app = App::new("rsbkb")
        .version("0.1.0")
        .author("Raphael Rigo <devel@syscall.eu>")
        .about("Rust BlackBag")
        .arg(Arg::with_name("tool")
                 .short("t")
                 .long("tool")
                 .default_value(&arg0)
                 .possible_values(&["unhex", "unhex2", "hex", "d64", "b64", "urldec", "urlenc", "xor", "crc32", "crc16", "slice"])
                 .takes_value(true)
                 .requires_if("slice", "value")
                 .help("Tool to run"))
        .arg(Arg::with_name("xorkey")
                 .short("x")
                 .long("xorkey")
                 .required_if("tool", "xor")
                 .takes_value(true)
                 .help("Xor key in hex format"))
        .arg(Arg::with_name("strict")
                 .short("s")
                 .long("strict")
                 .takes_value(false)
                 .help("strict decoding, error on invalid data"))
        .arg(Arg::with_name("value")
                 .required(false)
                 .help("input value, reads from stdin in not present"))
        .arg(Arg::with_name("start")
                 .required_if("tool", "slice")
                 .help("start from slice"))
        .arg(Arg::with_name("end")
                 .required(false)
                 .help("start from slice"));
    let matches = app.clone().get_matches();

    let operation = match matches.value_of("tool").unwrap() {
        "unhex" => Operation::HexDecode,
        "unhex2" => Operation::HexDecodeAll,
        "hex" => Operation::HexEncode,
        "d64" => Operation::B64Decode,
        "b64" => Operation::B64Encode,
        "urldec" => Operation::UrlDecode,
        "urlenc" => Operation::UrlEncode,
        "xor" => Operation::Xor,
        "crc16" => Operation::Crc16,
        "crc32" => Operation::Crc32,
        "slice" => Operation::Slice,
        _ => { &app.print_help(); println!(""); return;},
        };

    let mut inputval = vec![];

    /* No args, read from stdin */
    if !matches.is_present("value") {
        io::stdin().read_to_end(&mut inputval).expect("Reading stdin failed");
    } else {
        inputval = matches.value_of("value").unwrap().as_bytes().to_vec();
    }

    let res = process(matches, operation, inputval);

    let mut stdout = io::stdout();
    stdout.write(&res).expect("Write failed");
}
