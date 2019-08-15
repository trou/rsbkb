use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
extern crate hex;

fn usage(args: Vec<String>) {
    println!("Usage: {} [todecode]", args[0]);
}

fn main() {
    let args: Vec<_>= env::args().collect();

    /* No args, read from stdin */
    let mut hexval = String::new();
    if &args.len() < &2 {
        io::stdin().read_to_string(&mut hexval).expect("Reading stdin failed");
        hexval = hexval.trim().to_string();
    } else {
        if args[1] == "-h" {
            usage(args);
            return;
        }
        hexval = args[1].clone(); 
    }

    let decoded = hex::decode(hexval).expect("Decoding hex failed");
    let mut stdout = io::stdout();
    stdout.write(&decoded).expect("Write failed");
}
