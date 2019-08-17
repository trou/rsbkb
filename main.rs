use std::env;
use std::path::Path;
use std::io;
use std::io::Read;
use std::io::Write;
extern crate hex;
extern crate base64;
extern crate percent_encoding;

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

fn usage(program: String) {
    println!("Usage: {} [todecode]", program);
}

fn hex_decode(hexval: Vec<u8>) -> Vec<u8> {
    let decoded = hex::decode(hexval).expect("Decoding hex failed");
    return decoded;
}

fn b64_decode(b64val: Vec<u8>) -> Vec<u8> {
    let trimmed : Vec<u8> = b64val.trim().into();
    let decoded = base64::decode(&trimmed).expect("Decoding base64 failed");
    return decoded;
}

fn b64_encode(val: Vec<u8>) -> Vec<u8> {
    let encoded = base64::encode(&val);
    return encoded.as_bytes().to_vec();
}

fn hex_encode(val: Vec<u8>) -> Vec<u8> {
    let encoded = hex::encode(val);
    return encoded.as_bytes().to_vec();
}

fn url_decode(urlval: Vec<u8>) -> Vec<u8> {
    let trimmed : Vec<u8> = urlval.trim().into();
    let decoded = percent_encoding::percent_decode(&trimmed).decode_utf8_lossy();
    return decoded.as_bytes().to_vec();
}

fn main() {
    let args: Vec<_>= env::args().collect();

    if &args.len() < &1 {
        println!("No argv[0] !");
        return;
    }

    let arg0 = Path::new(&args[0]).file_name();
    let arg0 = match arg0 {
        Some(a) => a.to_str().unwrap().to_string(),
        None => panic!("No arg0"),
    };
    let operation = match arg0.as_ref() {
        "unhex" => hex_decode,
        "hex" => hex_encode,
        "d64" => b64_decode,
        "b64" => b64_encode,
        "urldec" => url_decode,
        _ => panic!("Unknown operation {}", arg0),
        };

    /* No args, read from stdin */
    let mut inputval = vec![];
    if &args.len() < &2 {
        io::stdin().read_to_end(&mut inputval).expect("Reading stdin failed");
    } else {
        let arg1 = args[1].clone();
        if arg1 == "-h" {
            usage(arg0);
            return;
        }
        inputval = arg1.as_bytes().to_vec();
    }

    let res = operation(inputval);

    let mut stdout = io::stdout();
    stdout.write(&res).expect("Write failed");
}
