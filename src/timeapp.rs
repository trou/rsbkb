use crate::applet::{Applet, FromStrWithRadix};
use anyhow::{Context, Result};
use clap::{arg, Command};
use std::convert::TryFrom;
use time::{format_description, Duration, OffsetDateTime, UtcDateTime, UtcOffset};

/*
    Decode a numeric timestamp in Epoch seconds format to a human-readable timestamp.

    An Epoch timestamp (1-10 digits) is an integer that counts the number of seconds since Jan 1
    1970.

    Useful values for ranges (all Jan-1 00:00:00):
      1970: 0
      2015: 1420070400
      2025: 1735689600
      2030: 1900000000
*/
fn decode_epoch_seconds(ts: i64) -> Result<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(ts).with_context(|| "Could not decode as epoch")
}

/* Decode epoch date with more precision */
fn decode_epoch_subseconds(ts: i64, resolution: i64) -> Result<OffsetDateTime> {
    let micros: i32 = i32::try_from((ts % resolution) * (1_000_000_000 / resolution)).unwrap();
    let unix = OffsetDateTime::from_unix_timestamp(ts / resolution);
    if let Ok(date) = unix {
        Ok(date + Duration::new(0, micros))
    } else {
        unix.with_context(|| "Could not decode as epoch")
    }
}

/*
    Decode a numeric timestamp in Windows FileTime format to a human-readable timestamp.

    A Windows FileTime timestamp (18 digits) is a 64-bit value that represents the number of
    100-nanosecond intervals since 12:00AM Jan 1 1601 UTC.

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

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum TimeEncoding {
    #[default]
    UnixSecond = 1,
    UnixCentiSecond = 100,
    UnixMilliSecond = 1000,
    UnixMicroSecond = 1000000,
    UnixNanoSecond = 1000000000,
    FILETIME,
    Chrome,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum TimeFormats {
    #[default]
    Iso8601,
    Rfc2822,
    Rfc3339,
}

pub struct TsEncApplet {
    encoding_type: TimeEncoding,
    input_format: TimeFormats,
}

impl Applet for TsEncApplet {
    fn command(&self) -> &'static str {
        "tsenc"
    }
    fn description(&self) -> &'static str {
        "timestamp encoder"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(
                arg!(-i --"input-format" [input] "input format")
                    .value_parser(clap::builder::EnumValueParser::<TimeFormats>::new())
                    .default_value("iso8601"),
            )
            .arg(
                arg!(-t --type [type] "type of timestamp to use for encoding")
                    .value_parser(clap::builder::EnumValueParser::<TimeEncoding>::new())
                    .default_value("unix-second"),
            )
            .arg(arg!([value]  "input value, reads from stdin if not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            encoding_type: TimeEncoding::UnixSecond,
            input_format: TimeFormats::Iso8601,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            encoding_type: args.get_one::<TimeEncoding>("type").unwrap().clone(),
            input_format: args.get_one::<TimeFormats>("input-format").unwrap().clone(),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let val_from_utf8 = String::from_utf8(val).context("Could not parse input as utf8")?;
        let val_str = val_from_utf8.as_str();

        let t = match self.input_format {
            TimeFormats::Iso8601 => UtcDateTime::parse(
                val_str,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ),
            TimeFormats::Rfc2822 => {
                UtcDateTime::parse(val_str, &time::format_description::well_known::Rfc2822)
            }
            TimeFormats::Rfc3339 => {
                UtcDateTime::parse(val_str, &time::format_description::well_known::Rfc3339)
            }
        }
        .context("Could not parse time")?;
        let res = match self.encoding_type {
            TimeEncoding::UnixSecond => (t - UtcDateTime::UNIX_EPOCH).whole_seconds() as i128,
            TimeEncoding::UnixCentiSecond => {
                (t - UtcDateTime::UNIX_EPOCH).whole_milliseconds() / 10
            }
            TimeEncoding::UnixMilliSecond => (t - UtcDateTime::UNIX_EPOCH).whole_milliseconds(),
            TimeEncoding::UnixMicroSecond => (t - UtcDateTime::UNIX_EPOCH).whole_microseconds(),
            TimeEncoding::UnixNanoSecond => (t - UtcDateTime::UNIX_EPOCH).whole_nanoseconds(),
            TimeEncoding::FILETIME => {
                ((t - UtcDateTime::UNIX_EPOCH).whole_nanoseconds() / 100) + 116_444_736_000_000_000
            }
            TimeEncoding::Chrome => {
                ((t - UtcDateTime::UNIX_EPOCH).whole_nanoseconds() / 1000) + 11_644_473_600_000_000
            }
        };
        Ok(format!("{}", res).as_bytes().to_vec())
    }
}
pub struct TsDecApplet {
    local: bool,
    verbose: bool,
}

impl Applet for TsDecApplet {
    fn command(&self) -> &'static str {
        "tsdec"
    }
    fn description(&self) -> &'static str {
        "timestamp decoder"
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-l --local  "show time in local time zone"))
            .arg(arg!(-v --verbose "show which type of timestamp was used for decoding"))
            .arg(arg!([value]  "input value, reads from stdin if not present"))
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            local: false,
            verbose: false,
        })
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {
            local: args.get_flag("local"),
            verbose: args.get_flag("verbose"),
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
        let (ts, type_str) = match (ts_len, ts_int) {
            (10, _) => (decode_epoch_seconds(ts_int), "Seconds since Epoch"),
            (12, _) =>
            /* Epoch centiseconds */
            {
                (
                    decode_epoch_subseconds(ts_int, 100),
                    "Centiseconds since Epoch",
                )
            }
            (13, _) =>
            /* Epoch milliseconds */
            {
                (
                    decode_epoch_subseconds(ts_int, 1000),
                    "Milliseconds since Epoch",
                )
            }
            (16, _) =>
            /* Epoch microseconds */
            {
                (
                    decode_epoch_subseconds(ts_int, 1_000_000),
                    "Microseconds since Epoch",
                )
            }
            (17, _) =>
            /* Chrome/WebKit timestamp: microseconds since 1601-01-01 */
            {
                (
                    decode_windows_filetime(ts_int * 10),
                    "Chrome/WebKit timestamp",
                )
            }
            (18, _) =>
            /* Windows FILETIME */
            {
                (decode_windows_filetime(ts_int), "Windows FILETIME")
            }
            _ => (decode_epoch_seconds(ts_int), "Seconds since Epoch"),
        };
        let tse = ts.with_context(|| "Could not convert timestamp")?;
        if self.verbose {
            eprintln!("Used format: {}", type_str);
        }
        let ts = if self.local {
            let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
            tse.to_offset(offset)
        } else {
            tse
        };
        let date_str = ts
            .format(&format_description::well_known::Rfc3339)
            .with_context(|| "Date formatting failed")?;
        Ok(date_str.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::TimeEncoding::*;
    use super::TimeFormats::*;
    use super::*;

    fn run_decode(app: &TsDecApplet, ts: &str) -> String {
        String::from_utf8(app.process_test(ts.as_bytes().to_vec())).unwrap()
    }

    #[test]
    fn test_tsdec_verbose_cli_stdin() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsdec", "-v"])
            .write_stdin("1")
            .assert()
            .stdout("1970-01-01T00:00:01Z")
            .stderr("Used format: Seconds since Epoch\n")
            .success();
    }

    #[test]
    fn test_tsdec_decimal() {
        let ts = TsDecApplet {
            local: false,
            verbose: false,
        };
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
    fn test_tsdec_hex() {
        let ts = TsDecApplet {
            local: false,
            verbose: false,
        };
        assert_eq!(run_decode(&ts, "0x0"), "1970-01-01T00:00:00Z");
        assert_eq!(run_decode(&ts, "0x1"), "1970-01-01T00:00:01Z");
    }

    fn run_encode(app: &TsEncApplet, date: &str) -> String {
        String::from_utf8(app.process_test(date.as_bytes().to_vec())).unwrap()
    }

    #[test]
    fn test_tsenc_cli_stdin() {
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsenc"])
            .write_stdin("1970-01-01T00:00:01Z")
            .assert()
            .stdout("1")
            .success();
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsenc", "-t", "filetime"])
            .write_stdin("1601-01-01T00:00:01Z")
            .assert()
            .stdout("10000000")
            .success();
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsenc", "-i", "iso8601"])
            .write_stdin("1970-01-01T00:00:01Z")
            .assert()
            .stdout("1")
            .success();
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsenc", "-i", "rfc2822"])
            .write_stdin("Sat, 12 Jun 1993 13:25:19 GMT")
            .assert()
            .stdout("739891519")
            .success();
        assert_cmd::Command::cargo_bin("rsbkb")
            .expect("Could not run binary")
            .args(&["tsenc", "-i", "rfc3339"])
            .write_stdin("1985-04-12T23:20:50.52Z")
            .assert()
            .stdout("482196050")
            .success();
    }

    #[test]
    fn test_tsenc() {
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: UnixCentiSecond,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "100"
        );
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: UnixMilliSecond,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "1000"
        );
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: UnixMicroSecond,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "1000000"
        );
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: UnixNanoSecond,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "1000000000"
        );
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: FILETIME,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "116444736010000000"
        );
        assert_eq!(
            run_encode(
                &TsEncApplet {
                    encoding_type: Chrome,
                    input_format: Iso8601,
                },
                "1970-01-01T00:00:01Z"
            ),
            "11644473601000000"
        );
    }
}
