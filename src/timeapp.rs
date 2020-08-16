use crate::applet::{FromStrWithRadix, Applet};
use time::{OffsetDateTime};

pub struct TimeApplet {}

impl Applet for TimeApplet {
    fn command(&self) -> &'static str { "tsdec" }
    fn description(&self) -> &'static str { "TimeStamp decode" }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Box<dyn Applet> {
        Box::new(Self { })
    }

    fn process(&self, val: Vec<u8>) -> Vec<u8> {
        let ts_str = String::from_utf8(val).unwrap();
        let ts = OffsetDateTime::from_unix_timestamp(i64::from_str_with_radix(ts_str.as_str()).unwrap());
        let date_str = ts.format("%F %T");
        return date_str.as_bytes().to_vec();
    }

}
