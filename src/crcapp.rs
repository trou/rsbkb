extern crate crc;
use crc::{crc16, crc32};
use crate::applet::Applet;

pub struct CRC16Applet {}

impl Applet for CRC16Applet {
    fn command(&self) -> &'static str { "crc16" }
    fn description(&self) -> &'static str { "compute CRC-16" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self { })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        format!("{:04x}", crc16::checksum_x25(&val)).as_bytes().to_vec()
    }

}


pub struct CRC32Applet {}

impl Applet for CRC32Applet {
    fn command(&self) -> &'static str { "crc32" }
    fn description(&self) -> &'static str { "compute CRC-32" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self { })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        format!("{:08x}", crc32::checksum_ieee(&val)).as_bytes().to_vec()
    }

}
