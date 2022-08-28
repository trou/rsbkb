use crate::applet::{Applet, FromStrWithRadix};
use crate::errors::{Result, ResultExt};
use clap::{arg, App, Command};
use std::convert::TryFrom;
use time::{format_description, Duration, OffsetDateTime, UtcOffset};

/*
    Decode a numeric timestamp in Epoch seconds format to a human-readable timestamp.

    An Epoch timestamp (1-10 digits) is an integer that counts the number of seconds since Jan 1 1970.

    Useful values for ranges (all Jan-1 00:00:00):
      1970: 0
      2015: 1420070400
      2025: 1735689600
      2030: 1900000000
*/
fn decode_epoch_seconds(ts: i64) -> Result<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(ts).chain_err(|| "Could not decode as epoch")
}

/* Decode epoch date with more precision */
fn decode_epoch_subseconds(ts: i64, resolution: i64) -> Result<OffsetDateTime> {
    let micros: i32 = i32::try_from((ts % resolution) * (1_000_000_000 / resolution)).unwrap();
    let unix = OffsetDateTime::from_unix_timestamp(ts / resolution);
    if let Ok(date) = unix {
        Ok(date + Duration::new(0, micros))
    } else {
        unix.chain_err(|| "Could not decode as epoch")
    }
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
fn decode_windows_filetime(ts: i64) -> Result<OffsetDateTime> {
    /* Shift to Unix Epoch */
    let shifted = ts - 116_444_736_000_000_000;
    decode_epoch_subseconds(shifted, 10_000_000)
}

pub struct TimeApplet {
    local: bool,
}
impl Applet for TimeApplet {
    fn command(&self) -> &'static str {
        "tsdec"
    }
    fn description(&self) -> &'static str {
        "TimeStamp decode"
    }

    fn clap_command(&self) -> App {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-l --local  "show time in local time zone"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self { local: false })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            local: args.is_present("local"),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let ts_str = String::from_utf8(val).unwrap();
        let ts_int = i64::from_str_with_radix(ts_str.as_str()).unwrap();
        let ts_len = if !ts_str.starts_with("0x") {
            // if the string is in decimal, return the number of digits
            ts_str.len()
        } else {
            // if in hex, compute length using log
            let ts_f: f64 = ts_int as f64;
            (ts_f.log10() as usize) + 1
        };
        let ts = match (ts_len, ts_int) {
            (10, _) => decode_epoch_seconds(ts_int),
            (12, _) =>
            /* Epoch centiseconds */
            {
                decode_epoch_subseconds(ts_int, 100)
            }
            (13, _) =>
            /* Epoch milliseconds */
            {
                decode_epoch_subseconds(ts_int, 1000)
            }
            (16, _) =>
            /* Epoch microseconds */
            {
                decode_epoch_subseconds(ts_int, 1_000_000)
            }
            (18, _) =>
            /* Windows FILETIME */
            {
                decode_windows_filetime(ts_int)
            }
            _ => decode_epoch_seconds(ts_int),
        }
        .chain_err(|| "Could not convert timestamp")?;
        let ts = if self.local {
            let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
            ts.to_offset(offset)
        } else {
            ts
        };
        let date_str = ts
            .format(&format_description::well_known::Rfc3339)
            .chain_err(|| "Date formatting failed")?;
        return Ok(date_str.as_bytes().to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_decode(app: &TimeApplet, ts: &str) -> String {
        String::from_utf8(app.process_test(ts.as_bytes().to_vec())).unwrap()
    }

    #[test]
    fn test_decimal() {
        let ts = TimeApplet { local: false };
        assert_eq!(run_decode(&ts, "0"), "1970-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "1420070400"), "2015-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "142007040000"), "2015-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "1420070400000"), "2015-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "1420070400000000"), "2015-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "142007040001"), "2015-01-01T00:00:00.01Z");
        assert_eq!(run_decode(&ts, "1420070400001"), "2015-01-01T00:00:00.001Z");
        assert_eq!(
            run_decode(&ts, "1420070400000001"),
            "2015-01-01T00:00:00.000001Z"
        );
        assert_eq!(
            run_decode(&ts, "146424672000234122"),
            "2065-01-01T00:00:00.0234122Z"
        );
        assert_eq!(
            run_decode(&ts, "000000000000000000"),
            "1601-01-01T00:00:00Z"
        );
    }

    #[test]
    fn test_hex() {
        let ts = TimeApplet { local: false };
        assert_eq!(run_decode(&ts, "0x0"), "1970-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "0x1"), "1970-01-01T00:00:01Z");
    }
}
