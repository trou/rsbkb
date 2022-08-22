use crate::applet::Applet;
use crate::errors::Result;

pub struct EntropyApplet {}

fn entropy(val: &[u8]) -> f64 {
    if val.is_empty() {
        return 0.0;
    }
    /* Compute how many times each value appears */
    let mut counts: [i64; 256] = [0; 256];
    for v in val.iter() {
        counts[*v as usize] += 1
    }

    let mut res: f64 = 0.0;
    let len: f64 = val.len() as f64;

    /* Compute entropy */
    for count in counts.iter() {
        if *count > 0 {
            let p = (*count as f64) / len;
            res -= p * p.log(256.0)
        }
    }
    res
}

impl Applet for EntropyApplet {
    fn command(&self) -> &'static str {
        "entropy"
    }
    fn description(&self) -> &'static str {
        "compute file entropy"
    }

    fn new() -> Box<dyn Applet> {
        Box::new(Self {})
    }

    fn parse_args(&self, _args: &clap::ArgMatches) -> Result<Box<dyn Applet>> {
        Ok(Box::new(Self {}))
    }

    fn process(&self, val: Vec<u8>) -> Result<Vec<u8>> {
        Ok(format!("{:.3}", entropy(val.as_slice()))
            .as_bytes()
            .to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ent(app: &EntropyApplet, val: Vec<u8>) -> String {
        String::from_utf8(app.process(val)).unwrap()
    }

    #[test]
    fn test() {
        let ent = EntropyApplet {};
        assert_eq!(run_ent(&ent, Vec::new()), "0.000");
        assert_eq!(run_ent(&ent, vec![1, 2, 3, 4]), "0.250");
        let mut all_bytes: Vec<u8> = Vec::with_capacity(256);
        for i in 0..255 {
            all_bytes.push(i as u8)
        }
        assert_eq!(run_ent(&ent, all_bytes), "0.999");
    }
}
