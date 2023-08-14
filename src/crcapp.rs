extern crate crc;
use crate::applet::Applet;
use anyhow::{bail, Result};
use clap::{arg, Command};
use crc::*;
use std::process;

pub struct CRC16Applet {}

impl Applet for CRC16Applet {
    fn command(&self) -> &'static str {
        "crc16"
    }
    fn description(&self) -> &'static str {
        "compute CRC-16"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
        Ok(format!("{:04x}", CRC16.checksum(&val)).as_bytes().to_vec())
    }
}

pub struct CRC32Applet {}

impl Applet for CRC32Applet {
    fn command(&self) -> &'static str {
        "crc32"
    }
    fn description(&self) -> &'static str {
        "compute CRC-32"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        Ok(format!("{:08x}", CRC32.checksum(&val)).as_bytes().to_vec())
    }
}

pub struct CRCApplet {
    crctype: String,
}

macro_rules! algs {
    ( $ident:expr; $size:tt; $( $x:expr ),* ) => {
        match $ident {
            $( stringify!($x) => Crc::<$size>::new(&$x), )*
            _ => { bail!("Unknown CRC algorithm.") } ,
        }
    }
}

impl Applet for CRCApplet {
    fn command(&self) -> &'static str {
        "crc"
    }
    fn description(&self) -> &'static str {
        "flexible CRC computation"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {
            crctype: "lol".to_string(),
        })
    }

    fn clap_command(&self) -> Command {
        Command::new(self.command())
            .about(self.description())
            .arg(arg!(-l --list  "List supported CRC algorithms"))
            .arg(arg!([type] "CRC type to compute").required_unless_present("list"))
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        if args.contains_id("list") {
            println!("Supported algorithms:");
            println!(
                "CRC16:
    CRC_16_ARC, CRC_16_CDMA2000, CRC_16_CMS, CRC_16_DDS_110,
    CRC_16_DECT_R, CRC_16_DECT_X, CRC_16_DNP, CRC_16_EN_13757, CRC_16_GENIBUS, CRC_16_GSM,
    CRC_16_IBM_3740, CRC_16_IBM_SDLC, CRC_16_ISO_IEC_14443_3_A, CRC_16_KERMIT, CRC_16_LJ1200,
    CRC_16_MAXIM_DOW, CRC_16_MCRF4XX, CRC_16_MODBUS, CRC_16_NRSC_5, CRC_16_OPENSAFETY_A,
    CRC_16_OPENSAFETY_B, CRC_16_PROFIBUS, CRC_16_RIELLO, CRC_16_SPI_FUJITSU, CRC_16_T10_DIF,
    CRC_16_TELEDISK, CRC_16_TMS37157, CRC_16_UMTS, CRC_16_USB, CRC_16_XMODEM"
            );
            println!(
                "CRC32:
    CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2, CRC_32_CD_ROM_EDC, CRC_32_CKSUM,
    CRC_32_ISCSI, CRC_32_ISO_HDLC, CRC_32_JAMCRC, CRC_32_MPEG_2, CRC_32_XFER"
            );
            println!(
                "CRC64:
    CRC_64_ECMA_182, CRC_64_GO_ISO, CRC_64_WE, CRC_64_XZ"
            );
            println!("\nSee https://docs.rs/crc/2.1.0/crc/ for more info");
            process::exit(0);
        }
        Ok(Box::new(Self {
            crctype: args.get_one::<String>("type").unwrap().to_string(),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let alg_name: &str = self.crctype.as_str();
        if self.crctype.contains("_16_") {
            let crc16 = algs!(alg_name; u16; CRC_16_ARC, CRC_16_CDMA2000, CRC_16_CMS, CRC_16_DDS_110,
                        CRC_16_DECT_R, CRC_16_DECT_X, CRC_16_DNP, CRC_16_EN_13757, CRC_16_GENIBUS, CRC_16_GSM,
                        CRC_16_IBM_3740, CRC_16_IBM_SDLC, CRC_16_ISO_IEC_14443_3_A, CRC_16_KERMIT, CRC_16_LJ1200,
                        CRC_16_MAXIM_DOW, CRC_16_MCRF4XX, CRC_16_MODBUS, CRC_16_NRSC_5, CRC_16_OPENSAFETY_A,
                        CRC_16_OPENSAFETY_B, CRC_16_PROFIBUS, CRC_16_RIELLO, CRC_16_SPI_FUJITSU, CRC_16_T10_DIF,
                        CRC_16_TELEDISK, CRC_16_TMS37157, CRC_16_UMTS, CRC_16_USB, CRC_16_XMODEM);
            return Ok(format!("{:04x}", crc16.checksum(&val)).as_bytes().to_vec());
        }
        if self.crctype.contains("_32_") {
            let crc32 = algs!(alg_name; u32; CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2,
                    CRC_32_CD_ROM_EDC, CRC_32_CKSUM, CRC_32_ISCSI, CRC_32_ISO_HDLC, CRC_32_JAMCRC,
                    CRC_32_MPEG_2, CRC_32_XFER);
            return Ok(format!("{:08x}", crc32.checksum(&val)).as_bytes().to_vec());
        }
        if self.crctype.contains("_64_") {
            let crc64 = algs!(alg_name; u64; CRC_64_ECMA_182, CRC_64_GO_ISO, CRC_64_WE, CRC_64_XZ);
            return Ok(format!("{:016x}", crc64.checksum(&val)).as_bytes().to_vec());
        }
        bail!("Unknown CRC algorithm");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let crc32 = CRC32Applet {};
        assert_eq!(
            "10cca4f1".as_bytes().to_vec(),
            crc32.process_test("toto".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_crc() {
        let crc = CRCApplet {
            crctype: "CRC_32_AIXM".to_string(),
        };
        assert_eq!(
            "fa83f52a".as_bytes().to_vec(),
            crc.process_test("toto".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_crc16() {
        let crc16 = CRC16Applet {};
        assert_eq!(
            "97a8".as_bytes().to_vec(),
            crc16.process_test("toto".as_bytes().to_vec())
        );
    }
}
