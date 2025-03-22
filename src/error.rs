use std::{borrow::Cow, io::IsTerminal};

use crate::{
    arg::ArgSpec,
    args::{Args, Metadata},
};

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    UnexpectedArg {
        metadata: Metadata,
        arg: String,
    },
    ParseArgError {
        arg: ArgSpec,
        value: String,
        reason: String,
    },
    MissingArgValue {
        arg: ArgSpec,
    },
    Other(Box<dyn std::fmt::Display>),
}

impl Error {
    pub fn check_unexpected_arg(args: &Args) -> Result<()> {
        if let Some(unexpected_arg) = args.next_raw_arg_value() {
            Err(Error::UnexpectedArg {
                metadata: args.metadata(),
                arg: unexpected_arg.to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn to_string(&self, is_terminal: bool) -> String {
        let mut fmt = Formatter::new(is_terminal);
        match self {
            Error::UnexpectedArg { metadata, arg } => {
                fmt.format_unexpected_arg(*metadata, arg);
            }
            Error::ParseArgError { arg, value, reason } => {
                todo!()
            }
            Error::MissingArgValue { arg } => {
                todo!()
            }
            Error::Other(e) => {
                fmt.text = e.to_string();
            }
        }
        fmt.text
    }
}

impl<T: 'static + std::fmt::Display> From<T> for Error {
    fn from(error: T) -> Self {
        Self::Other(Box::new(error))
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(std::io::stderr().is_terminal()))
    }
}

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

        if let Some(help_flag_name) = metadata.help_flag_name {
            self.write(&format!(
                "\nTry '{}' for more information.",
                self.bold(&format!("--{help_flag_name}"))
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
        assert_eq!(e.to_string(false), "unexpected argument '--foo' found");

        // Error with `--help`.
        let mut args = Args::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = Some("help");
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(
            e.to_string(false),
            r#"unexpected argument '--foo' found
Try '--help' for more information."#
        );
    }
}
