use std::io::IsTerminal;

use crate::{Arg, Args, Metadata, Opt, args::Taken, formatter::Formatter};

/// Possible errors.
///
/// Note that this enum is intended to be used only as a top-level error and
/// deliberately does not implement the [`std::error::Error`] and [`std::fmt::Display`] traits.
///
/// Additionally, any external errors that implement [`std::fmt::Display`] can be converted into this error.
#[allow(missing_docs)]
pub enum Error {
    UnexpectedArg { metadata: Metadata, name: String },
    UndefinedCommand { metadata: Metadata, name: String },
    MissingCommand { metadata: Metadata },
    ParseArgError { arg: Box<Arg>, reason: String },
    MissingArg { arg: Box<Arg> },
    ParseOptError { opt: Box<Opt>, reason: String },
    MissingOpt { opt: Box<Opt> },
    Other(Box<dyn std::fmt::Display>),
}

impl Error {
    pub(crate) fn check_command_error(args: &Args) -> Result<(), Error> {
        let Some(Taken::Cmd(cmd)) = args.log().last() else {
            return Ok(());
        };
        if cmd.is_present() {
            return Ok(());
        }
        if let Some((_, name)) = args.remaining_args().next() {
            Err(Self::UndefinedCommand {
                metadata: args.metadata(),
                name: name.to_owned(),
            })
        } else {
            Err(Self::MissingCommand {
                metadata: args.metadata(),
            })
        }
    }

    pub(crate) fn check_unexpected_arg(args: &Args) -> Result<(), Error> {
        if let Some(unexpected_arg) = args.next_raw_arg_value() {
            Err(Error::UnexpectedArg {
                metadata: args.metadata(),
                name: unexpected_arg.to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn to_string(&self, is_terminal: bool) -> String {
        let mut fmt = Formatter::new(is_terminal);
        match self {
            Error::UnexpectedArg {
                metadata,
                name: arg,
            } => {
                Self::format_unexpected_arg(&mut fmt, *metadata, arg);
            }
            #[expect(unused_variables)]
            Error::UndefinedCommand { metadata, name } => {
                todo!()
            }
            #[expect(unused_variables)]
            Error::MissingCommand { metadata } => {
                todo!()
            }
            #[expect(unused_variables)]
            Error::ParseArgError { arg, reason } => {
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
