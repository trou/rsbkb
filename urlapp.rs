use crate::applet::Applet;
use crate::applet::SliceExt;

pub struct UrlEncApplet {}

impl Applet for UrlEncApplet {
    fn command(&self) -> &'static str { "urlenc" }
    fn description(&self) -> &'static str { "URL encode" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {}) 
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self { })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        let encoded = percent_encoding::percent_encode(&val, percent_encoding::NON_ALPHANUMERIC).to_string();
        return encoded.as_bytes().to_vec();
    }

}


pub struct UrlDecApplet {}

impl Applet for UrlDecApplet {
    fn command(&self) -> &'static str { "urldec" }
    fn description(&self) -> &'static str { "URL decode" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {}) 
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self { })
    }

    fn process(&self, urlval: Vec<u8>) -> Vec<u8> {
        let trimmed : Vec<u8> = urlval.trim().into();
        let decoded: Vec<u8> = percent_encoding::percent_decode(&trimmed).collect();
        return decoded;
    }

}


