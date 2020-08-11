use clap::{App, SubCommand};


pub trait Applet {
    fn command(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn parse_args(&self, args: &clap::ArgMatches) -> Box<dyn Applet>;

    fn subcommand(&self) -> App {
        SubCommand::with_name(self.command()).about(self.description())
             .arg_from_usage("[value] 'input value, reads from stdin in not present'")
    }

    fn arg_or_stdin(&self) -> Option<&'static str> { Some("value") }

    fn process(&self, val: Vec<u8>) -> Vec<u8>;

    fn new() -> Box<dyn Applet> where Self: Sized;
}

/* Helper to trim whitespace */
pub trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b' ' || *c == b'\n' || *c == b'\r'
        }

        fn is_not_whitespace(c: &u8) -> bool {
            !is_whitespace(c)
        }

        if let Some(first) = self.iter().position(is_not_whitespace) {
            if let Some(last) = self.iter().rposition(is_not_whitespace) {
                &self[first..last + 1]
            } else {
                unreachable!();
            }
        } else {
            &[]
        }
    }
}
