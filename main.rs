use std::env;
use std::path::Path;
use std::io;
use std::io::Read;
use std::io::Write;
extern crate hex;

fn usage(program: String) {
    println!("Usage: {} [todecode]", program);
}

fn hex_decode(hexval: Vec<u8>) -> Vec<u8> {
    let decoded = hex::decode(hexval).expect("Decoding hex failed");
    return decoded;
}

fn hex_encode(val: Vec<u8>) -> Vec<u8> {
    let encoded = hex::encode(val);
    return encoded.as_bytes().to_vec();
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
        _ => panic!("Unknown operation"),
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
