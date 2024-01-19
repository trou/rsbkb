use crate::applet::Applet;
use anyhow::{Context, Result};
use clap::{arg, Command};
use std::fs;

pub struct XorApplet {
    key_bytes: Vec<u8>,
}

impl Applet for XorApplet {
    fn command(&self) -> &'static str {
        "xor"
    }
    fn description(&self) -> &'static str {
        "xor value"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(
                arg!(-x --xorkey <KEY>  "Xor key in hex format")
                    .required_unless_present("keyfile")
                    .conflicts_with("keyfile"),
            )
            .arg(arg!(-f --keyfile <keyfile>  "File to use as key"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { key_bytes: vec![] })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        let key_bytes = if args.contains_id("xorkey") {
            hex::decode(args.get_one::<String>("xorkey").unwrap().replace(' ', ""))
                .with_context(|| "Xor key decoding failed")?
        } else {
            fs::read(args.get_one::<String>("keyfile").unwrap())
                .with_context(|| "Could not read keyfile")?
        };
        Ok(Box::new(Self { key_bytes }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let inf_key = self.key_bytes.iter().cycle(); // Iterate endlessly over key bytes
        return Ok(val.iter().zip(inf_key).map(|(x, k)| x ^ k).collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hex_key_cli() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["xor", "-x", "41", "AAAA"])
            .assert()
            .stdout(&b"\0\0\0\0"[..])
            .success();
    }

    #[test]
    fn test_key_file_cli_stdin() {
        let rand_key: [u8; 17] = [
            0x7f, 0x5a, 0x88, 0x7b, 0xe8, 0x81, 0xd6, 0x5e, 0x39, 0xf4, 0x7e, 0x25, 0xf2, 0x05,
            0xdc, 0x22, 0x86,
        ];
        let zero_data = [0u8; 32];

        let mut tmpkey = tempfile::NamedTempFile::new().unwrap();
        tmpkey.write(&rand_key.clone()).unwrap();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["xor", "-f", tmpkey.path().to_str().expect("Could not get path as str")])
            .write_stdin(zero_data)
            .assert()
            .stdout(&b"\x7fZ\x88{\xE8\x81\xD6^9\xF4~%\xF2\x05\xDC\"\x86\x7fZ\x88{\xE8\x81\xD6^9\xF4~%\xF2\x05\xDC"[..])
            .success();

        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&[
                "xor",
                "-f",
                tmpkey.path().to_str().expect("Could not get path as str"),
            ])
            .write_stdin(rand_key)
            .assert()
            .stdout(&b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"[..])
            .success();
    }

    #[test]
    fn test_simple() {
        let data = vec![1, 0x55, 0xAA, 0xFF, 0];
        let x = XorApplet {
            key_bytes: data.clone(),
        };
        assert_eq!(x.process_test(vec![0, 0, 0, 0, 0]), data);
        assert_eq!(
            x.process_test(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            vec![0xFE, 0xAA, 0x55, 0, 0xFF]
        );
        assert_eq!(x.process_test(vec![0]), vec![1]);
        assert_eq!(
            x.process_test(vec![0, 0, 0, 0, 0, 0]),
            vec![1, 0x55, 0xAA, 0xFF, 0, 1]
        );
    }
}
