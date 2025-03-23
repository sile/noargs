use std::io::IsTerminal;

use crate::{
    arg::ArgSpec,
    args::{Args, Metadata},
    formatter::Formatter,
    opt::Opt,
};

pub enum Error {
    UnexpectedArg {
        metadata: Metadata,
        arg: String,
    },
    ParseArgError {
        arg: ArgSpec, // TODO: Arg
        value: String,
        reason: String,
    },
    MissingArg {
        arg: ArgSpec, // TODO: Arg
    },
    ParseOptError {
        opt: Opt,
        reason: String,
    },
    MissingOpt {
        opt: Opt,
    },
    Other(Box<dyn std::fmt::Display>),
}

impl Error {
    pub(crate) fn check_undefined_command(args: &Args) -> Result<(), Error> {
        todo!()
    }

    pub(crate) fn check_unexpected_arg(args: &Args) -> Result<(), Error> {
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
                Self::format_unexpected_arg(&mut fmt, *metadata, arg);
            }
            #[expect(unused_variables)]
            Error::ParseArgError { arg, value, reason } => {
                todo!()
            }
            #[expect(unused_variables)]
            Error::MissingArg { arg } => {
                todo!()
            }
            #[expect(unused_variables)]
            Error::ParseOptError { opt, reason } => todo!(),
            #[expect(unused_variables)]
            Error::MissingOpt { opt } => todo!(),
            Error::Other(e) => {
                fmt.write(&e.to_string());
            }
        }
        fmt.finish()
    }

    fn format_unexpected_arg(fmt: &mut Formatter, metadata: Metadata, arg: &str) {
        fmt.write(&format!("unexpected argument '{}' found", fmt.bold(arg)));
        Self::write_help_line(fmt, metadata);
    }

    fn write_help_line(fmt: &mut Formatter, metadata: Metadata) {
        if let Some(help_flag_name) = metadata.help_flag_name {
            fmt.write(&format!(
                "\nTry '{}' for more information.",
                fmt.bold(&format!("--{help_flag_name}"))
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_unexpected_arg() {
        // No error.
        let args = Args::new(["noargs"].iter().map(|a| a.to_string()));
        assert!(Error::check_unexpected_arg(&args).is_ok());

        // Error without `--help`.
        let mut args = Args::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(e.to_string(false), "unexpected argument '--foo' found");

        // Error with `--help`.
        let args = Args::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(
            e.to_string(false),
            r#"unexpected argument '--foo' found
Try '--help' for more information."#
        );
    }
}
