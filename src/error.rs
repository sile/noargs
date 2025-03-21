use std::{borrow::Cow, io::IsTerminal};

use crate::args::{Args, Metadata};

pub enum Error {
    UnexpectedArg { metadata: Metadata, arg: String },
}

impl Error {
    pub fn check_unexpected_arg(args: &Args) -> Result<(), Error> {
        if let Some(unexpected_arg) = args.next_raw_arg_value() {
            Err(Error::UnexpectedArg {
                metadata: args.metadata(),
                arg: unexpected_arg.to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = Formatter::new(std::io::stderr().is_terminal());
        match self {
            Error::UnexpectedArg { metadata, arg } => fmt.format_unexpected_arg(*metadata, arg),
        }
        write!(f, "{}", fmt.text)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = Formatter::new(false);
        match self {
            Error::UnexpectedArg { metadata, arg } => fmt.format_unexpected_arg(*metadata, arg),
        }
        write!(f, "{}", fmt.text)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default)]
struct Formatter {
    text: String,
    is_terminal: bool,
}

impl Formatter {
    fn new(is_terminal: bool) -> Self {
        Self {
            text: String::new(),
            is_terminal,
        }
    }

    fn format_unexpected_arg(&mut self, metadata: Metadata, arg: &str) {
        self.write(&format!("unexpected argument '{}' found", self.bold(arg)));

        if let Some(help) = metadata.help_option_name {
            self.write(&format!(
                "\nTry '{}' for more information.",
                self.bold(&format!("--{}", help))
            ));
        }
    }

    fn write(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn bold<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("\x1b[1m{}\x1b[0m", s))
        } else {
            Cow::Borrowed(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_unexpected_arg() {
        // No error.
        let args = Args::new(["noargs"].iter().map(|a| a.to_string()));
        assert!(Error::check_unexpected_arg(&args).is_ok());

        // Error without `--help`.
        let args = Args::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(e.to_string(), "unexpected argument '--foo' found");

        // Error with `--help`.
        let mut args = Args::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_option_name = Some("help");
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(
            e.to_string(),
            r#"unexpected argument '--foo' found
Try '--help' for more information."#
        );
    }
}
