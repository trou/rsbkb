extern crate crc;
use crate::applet::Applet;
use anyhow::{bail, Result};
use clap::{arg, Command};
use crc::*;
use std::process;

const ALL_CRCS: [&str; 111] = [
    "CRC_3_GSM",
    "CRC_3_ROHC",
    "CRC_4_G_704",
    "CRC_4_INTERLAKEN",
    "CRC_5_EPC_C1G2",
    "CRC_5_G_704",
    "CRC_5_USB",
    "CRC_6_CDMA2000_A",
    "CRC_6_CDMA2000_B",
    "CRC_6_DARC",
    "CRC_6_GSM",
    "CRC_6_G_704",
    "CRC_7_MMC",
    "CRC_7_ROHC",
    "CRC_7_UMTS",
    "CRC_8_AUTOSAR",
    "CRC_8_BLUETOOTH",
    "CRC_8_CDMA2000",
    "CRC_8_DARC",
    "CRC_8_DVB_S2",
    "CRC_8_GSM_A",
    "CRC_8_GSM_B",
    "CRC_8_HITAG",
    "CRC_8_I_432_1",
    "CRC_8_I_CODE",
    "CRC_8_LTE",
    "CRC_8_MAXIM_DOW",
    "CRC_8_MIFARE_MAD",
    "CRC_8_NRSC_5",
    "CRC_8_OPENSAFETY",
    "CRC_8_ROHC",
    "CRC_8_SAE_J1850",
    "CRC_8_SMBUS",
    "CRC_8_TECH_3250",
    "CRC_8_WCDMA",
    "CRC_10_ATM",
    "CRC_10_CDMA2000",
    "CRC_10_GSM",
    "CRC_11_FLEXRAY",
    "CRC_11_UMTS",
    "CRC_12_CDMA2000",
    "CRC_12_DECT",
    "CRC_12_GSM",
    "CRC_12_UMTS",
    "CRC_13_BBC",
    "CRC_14_DARC",
    "CRC_14_GSM",
    "CRC_15_CAN",
    "CRC_15_MPT1327",
    "CRC_16_ARC",
    "CRC_16_CDMA2000",
    "CRC_16_CMS",
    "CRC_16_DDS_110",
    "CRC_16_DECT_R",
    "CRC_16_DECT_X",
    "CRC_16_DNP",
    "CRC_16_EN_13757",
    "CRC_16_GENIBUS",
    "CRC_16_GSM",
    "CRC_16_IBM_3740",
    "CRC_16_IBM_SDLC",
    "CRC_16_ISO_IEC_14443_3_A",
    "CRC_16_KERMIT",
    "CRC_16_LJ1200",
    "CRC_16_M17",
    "CRC_16_MAXIM_DOW",
    "CRC_16_MCRF4XX",
    "CRC_16_MODBUS",
    "CRC_16_NRSC_5",
    "CRC_16_OPENSAFETY_A",
    "CRC_16_OPENSAFETY_B",
    "CRC_16_PROFIBUS",
    "CRC_16_RIELLO",
    "CRC_16_SPI_FUJITSU",
    "CRC_16_T10_DIF",
    "CRC_16_TELEDISK",
    "CRC_16_TMS37157",
    "CRC_16_UMTS",
    "CRC_16_USB",
    "CRC_16_XMODEM",
    "CRC_17_CAN_FD",
    "CRC_21_CAN_FD",
    "CRC_24_BLE",
    "CRC_24_FLEXRAY_A",
    "CRC_24_FLEXRAY_B",
    "CRC_24_INTERLAKEN",
    "CRC_24_LTE_A",
    "CRC_24_LTE_B",
    "CRC_24_OPENPGP",
    "CRC_24_OS_9",
    "CRC_30_CDMA",
    "CRC_31_PHILIPS",
    "CRC_32_AIXM",
    "CRC_32_AUTOSAR",
    "CRC_32_BASE91_D",
    "CRC_32_BZIP2",
    "CRC_32_CD_ROM_EDC",
    "CRC_32_CKSUM",
    "CRC_32_ISCSI",
    "CRC_32_ISO_HDLC",
    "CRC_32_MEF",
    "CRC_32_MPEG_2",
    "CRC_32_XFER",
    "CRC_40_GSM",
    "CRC_64_ECMA_182",
    "CRC_64_GO_ISO",
    "CRC_64_MS",
    "CRC_64_REDIS",
    "CRC_64_WE",
    "CRC_64_XZ",
    "CRC_82_DARC",
];

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
            .arg(
                arg!([type] "CRC type to compute. Use 'all' to compute all known algorithms.")
                    .required_unless_present("list"),
            )
            .arg(arg!([value]  "input value, reads from stdin in not present"))
    }

    fn parse_args(&self, args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        if args.get_flag("list") {
            println!("Supported algorithms:");
            println!("{}", ALL_CRCS.join("\n"));
            println!("\nSee https://docs.rs/crc/ for more info");
            process::exit(0);
        }
        Ok(Box::new(Self {
            crctype: args.get_one::<String>("type").unwrap().to_string(),
        }))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        let alg_name: &str = self.crctype.as_str();
        if alg_name == "all" {
            let mut res = String::new();
            for alg in ALL_CRCS.iter() {
                res.push_str(format!("{}: 0x{}\n", alg, self.do_crc(alg, &val)?.as_str()).as_str());
            }
            Ok(res.as_bytes().to_vec())
        } else {
            Ok(self.do_crc(alg_name, &val)?.as_bytes().to_vec())
        }
    }
}

impl CRCApplet {
    fn do_crc(&self, alg_name: &str, val: &[u8]) -> Result<String> {
        let alg_size: u8 = (*alg_name.split('_').collect::<Vec<&str>>().get(1).unwrap())
            .parse()
            .unwrap();
        match alg_size {
            0..=8 => {
                let crc8 = algs!(alg_name; u8;
                    CRC_3_GSM, CRC_3_ROHC, CRC_4_G_704, CRC_4_INTERLAKEN, CRC_5_EPC_C1G2, CRC_5_G_704, CRC_5_USB,
                    CRC_6_CDMA2000_A, CRC_6_CDMA2000_B, CRC_6_DARC, CRC_6_GSM, CRC_6_G_704, CRC_7_MMC, CRC_7_ROHC,
                    CRC_7_UMTS, CRC_8_AUTOSAR, CRC_8_BLUETOOTH, CRC_8_CDMA2000, CRC_8_DARC, CRC_8_DVB_S2, CRC_8_GSM_A,
                    CRC_8_GSM_B, CRC_8_HITAG, CRC_8_I_432_1, CRC_8_I_CODE, CRC_8_LTE, CRC_8_MAXIM_DOW,
                    CRC_8_MIFARE_MAD, CRC_8_NRSC_5, CRC_8_OPENSAFETY, CRC_8_ROHC, CRC_8_SAE_J1850, CRC_8_SMBUS,
                    CRC_8_TECH_3250, CRC_8_WCDMA);
                Ok(format!("{:02x}", crc8.checksum(val)))
            }
            9..=16 => {
                let crc16 = algs!(alg_name; u16;
                            CRC_10_ATM, CRC_10_CDMA2000, CRC_10_GSM, CRC_11_FLEXRAY, CRC_11_UMTS,
                            CRC_12_CDMA2000, CRC_12_DECT, CRC_12_GSM, CRC_12_UMTS, CRC_13_BBC,
                            CRC_14_DARC, CRC_14_GSM, CRC_15_CAN, CRC_15_MPT1327, CRC_16_ARC,
                            CRC_16_CDMA2000, CRC_16_CMS, CRC_16_DDS_110, CRC_16_DECT_R,
                            CRC_16_DECT_X, CRC_16_DNP, CRC_16_EN_13757, CRC_16_GENIBUS, CRC_16_GSM,
                            CRC_16_IBM_3740, CRC_16_IBM_SDLC, CRC_16_ISO_IEC_14443_3_A,
                            CRC_16_KERMIT, CRC_16_LJ1200, CRC_16_M17, CRC_16_MAXIM_DOW,
                            CRC_16_MCRF4XX, CRC_16_MODBUS, CRC_16_NRSC_5, CRC_16_OPENSAFETY_A,
                            CRC_16_OPENSAFETY_B, CRC_16_PROFIBUS, CRC_16_RIELLO,
                            CRC_16_SPI_FUJITSU, CRC_16_T10_DIF, CRC_16_TELEDISK, CRC_16_TMS37157,
                            CRC_16_UMTS, CRC_16_USB, CRC_16_XMODEM);
                Ok(format!("{:04x}", crc16.checksum(val)))
            }
            17..=32 => {
                let crc32 = algs!(alg_name; u32;
                        CRC_17_CAN_FD, CRC_21_CAN_FD, CRC_24_BLE, CRC_24_FLEXRAY_A, CRC_24_FLEXRAY_B, CRC_24_INTERLAKEN,
                        CRC_24_LTE_A, CRC_24_LTE_B, CRC_24_OPENPGP, CRC_24_OS_9, CRC_30_CDMA, CRC_31_PHILIPS, CRC_32_AIXM,
                        CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2, CRC_32_CD_ROM_EDC, CRC_32_CKSUM, CRC_32_ISCSI,
                        CRC_32_ISO_HDLC, CRC_32_JAMCRC, CRC_32_MEF, CRC_32_MPEG_2, CRC_32_XFER);
                Ok(format!("{:08x}", crc32.checksum(val)))
            }
            33..=64 => {
                let crc64 = algs!(alg_name; u64; CRC_40_GSM, CRC_64_ECMA_182, CRC_64_GO_ISO, CRC_64_MS, CRC_64_REDIS, CRC_64_WE, CRC_64_XZ);
                Ok(format!("{:016x}", crc64.checksum(val)))
            }
            65..=128 => {
                let crc128 = algs!(alg_name; u128; CRC_82_DARC);
                Ok(format!("{:032x}", crc128.checksum(val)))
            }
            _ => {
                bail!("Unknown CRC algorithm");
            }
        }
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
