use std::convert::TryFrom;
use crate::applet::{FromStrWithRadix, Applet};
use time::{OffsetDateTime, Duration};

pub struct TimeApplet {}

/*
    Decode a numeric timestamp in Epoch seconds format to a human-readable timestamp.

    An Epoch timestamp (1-10 digits) is an integer that counts the number of seconds since Jan 1 1970.

    Useful values for ranges (all Jan-1 00:00:00):
      1970: 0
      2015: 1420070400
      2025: 1735689600
      2030: 1900000000
*/
fn decode_epoch_seconds(ts: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ts)
}


/* Decode epoch date with more precision */
fn decode_epoch_subseconds(ts: i64, resolution: i64) -> OffsetDateTime {
    let micros: i32 = i32::try_from((ts%resolution)*(1_000_000_000/resolution)).unwrap();
    OffsetDateTime::from_unix_timestamp(ts/resolution)+Duration::new(0, micros)
}


/*
    Decode a numeric timestamp in Windows FileTime format to a human-readable timestamp.

    A Windows FileTime timestamp (18 digits) is a 64-bit value that represents the number of 100-nanosecond intervals
    since 12:00AM Jan 1 1601 UTC.

    Useful values for ranges (all Jan-1 00:00:00):
      1970: 116444736000000000
      2015: 130645440000000000
      2025: 133801632000000000
      2065: 146424672000000000
*/
fn decode_windows_filetime(ts: i64) -> OffsetDateTime {
    /* Shift to Unix Epoch */
    let shifted = ts-116444736000000000;
    decode_epoch_subseconds(shifted, 10_000_000)
}

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
        let ts_int = i64::from_str_with_radix(ts_str.as_str()).unwrap();
        let ts_len = if !ts_str.starts_with("0x") {
                // if the string is in decimal, return the number of digits
                ts_str.len() } else {
                // if in hex, compute length using log
                let ts_f : f64 = ts_int as f64;
                ((ts_f.ln()/10.0_f64.ln()) as usize)+1
                };
        let ts =
            match (ts_len, ts_int) {
                (10, _) => decode_epoch_seconds(ts_int),
                (12, _) => /* Epoch centiseconds */
                           decode_epoch_subseconds(ts_int, 100),
                (13, _) => /* Epoch milliseconds */
                            decode_epoch_subseconds(ts_int, 1000),
                (16, _) => /* Epoch microseconds */
                            decode_epoch_subseconds(ts_int, 1_000_000),
                (18, _) => /* Windows FILETIME */
                            decode_windows_filetime(ts_int),
                _ => decode_epoch_seconds(ts_int)
            };
        let date_str = ts.format("%F %T.%N");
        return date_str.as_bytes().to_vec();
    }
}
