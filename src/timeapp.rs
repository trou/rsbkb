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
        /*

            # Windows FileTime (18 digits)
            if 130645440000000000 <= timestamp <= 133801632000000000:  # 2015 <= ts <= 2025
                new_timestamp = decode_windows_filetime(timestamp)

            # .Net/C# DateTime ticks (18 digits)
            elif 635556672000000000 <= timestamp <= 638712864000000000:  # 2015 <= ts <= 2025
                new_timestamp = decode_datetime_ticks(timestamp)

            # WebKit (17 digits)
            elif 13064544000000000 <= timestamp <= 13380163200000000:  # 2015 <= ts <= 2025
                new_timestamp = decode_webkit(timestamp)

            # Epoch microseconds (16 digits)
            elif 1420070400000000 <= timestamp <= 1735689600000000:  # 2015 <= ts <= 2025
                new_timestamp = decode_epoch_microseconds(timestamp)

            # Epoch milliseconds (13 digits)
            elif 1420070400000 <= timestamp <= 1735689600000:  # 2015 <= ts <= 2025
                new_timestamp = decode_epoch_milliseconds(timestamp)

            # Epoch seconds (10 digits)
            elif 1420070400 <= timestamp <= 1735689600:  # 2015 <= ts <= 2025
                new_timestamp = decode_epoch_seconds(timestamp)

            # Mac Absolute Time (9 digits)
            elif 441763200 <= timestamp <= 757382400:  # 2015 <= ts <= 2025
                new_timestamp = decode_mac_absolute_time(timestamp)

        elif matches_float:
            timestamp = float(node.value)

            # Epoch seconds (10 digits)
            if 1420070400.0 <= timestamp <= 1735689600.0:  # 2015 <= ts <= 2025
                new_timestamp = decode_epoch_seconds(timestamp)

            # Mac Absolute Time (9 digits)
            elif 441763200.0 <= timestamp <= 757382400.0:  # 2015 <= ts <= 2025
                new_timestamp = decode_mac_absolute_time(timestamp)
                */
        let ts_str = String::from_utf8(val).unwrap();
        let ts_len = ts_str.len();
        let ts_int = i64::from_str_with_radix(ts_str.as_str()).unwrap();
        let ts =
            match (ts_len, ts_int) {
                // Windows FILETIME
                //(18, _) =>
                (10, 1420070400..=1735689600) => OffsetDateTime::from_unix_timestamp(ts_int),
                _ => OffsetDateTime::from_unix_timestamp(ts_int)
            };
        let date_str = ts.format("%F %T");
        return date_str.as_bytes().to_vec();
    }

}
