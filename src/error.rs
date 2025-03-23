use std::io::IsTerminal;

use crate::{Arg, Args, Metadata, Opt, OptSpec, args::Taken, formatter::Formatter};

/// Possible errors.
///
/// Note that this enum is intended to be used only as a top-level error and
/// deliberately does not implement the [`std::error::Error`] and [`std::fmt::Display`] traits.
///
/// Additionally, any external errors that implement [`std::fmt::Display`] can be converted into this error.
#[allow(missing_docs)]
pub enum Error {
    UnexpectedArg { metadata: Metadata, raw_arg: String },
    UndefinedCommand { metadata: Metadata, raw_arg: String },
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
        if let Some((_, raw_arg)) = args.remaining_args().next() {
            Err(Self::UndefinedCommand {
                metadata: args.metadata(),
                raw_arg: raw_arg.to_owned(),
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
                raw_arg: unexpected_arg.to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn to_string(&self, is_terminal: bool) -> String {
        let mut fmt = Formatter::new(is_terminal);
        let metadata = match self {
            Error::UnexpectedArg { metadata, raw_arg } => {
                fmt.write(&format!(
                    "unexpected argument '{}' found",
                    fmt.bold(raw_arg)
                ));
                *metadata
            }
            Error::UndefinedCommand { metadata, raw_arg } => {
                fmt.write(&format!("'{}' command is not defined", fmt.bold(raw_arg)));
                *metadata
            }
            Error::MissingCommand { metadata } => {
                fmt.write("command is not specified");
                *metadata
            }
            Error::ParseArgError { arg, reason } => {
                fmt.write(&format!(
                    "argument '{}' has an invalid value {:?}: {reason}",
                    fmt.bold(arg.spec().name),
                    arg.raw_value().unwrap_or_default()
                ));
                if let Some(metadata) = arg.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            #[expect(unused_variables)]
            Error::MissingArg { arg } => {
                todo!()
            }
            Error::ParseOptError { opt, reason } => {
                let name = match &**opt {
                    Opt::Short {
                        spec: OptSpec { short: Some(c), .. },
                        ..
                    } => format!("argument '{}'", fmt.bold(&format!("-{c}"))),
                    Opt::Env {
                        spec:
                            OptSpec {
                                env: Some(name), ..
                            },
                        ..
                    } => format!(
                        "environment variable '{}' for '{}'",
                        fmt.bold(name),
                        fmt.bold(&format!("--{}", opt.spec().name))
                    ),
                    _ => format!(
                        "argument '{}'",
                        fmt.bold(&format!(" --{}", opt.spec().name))
                    ),
                };
                fmt.write(&format!(
                    "{name} has an invalid value {:?}: {reason}",
                    opt.raw_value().unwrap_or_default()
                ));
                if let Some(metadata) = opt.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            #[expect(unused_variables)]
            Error::MissingOpt { opt } => todo!(),
            Error::Other(e) => {
                fmt.write(&e.to_string());
                return fmt.finish();
            }
        };
        Self::write_help_line(&mut fmt, metadata);
        fmt.finish()
    }

    fn write_help_line(fmt: &mut Formatter, metadata: Metadata) {
        if let Some(help_flag_name) = metadata.help_flag_name {
            fmt.write(&format!(
                "\n\nTry '{}' for more information.",
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
    use crate::{arg, cmd, opt};

    use super::*;

    #[test]
    fn unexpected_arg_error() {
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

    #[test]
    fn undefined_command_error() {
        let mut args = Args::new(["noargs", "baz"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        cmd("foo").take(&mut args);
        cmd("bar").take(&mut args);
        let e = args.finish().expect_err("error");
        assert_eq!(e.to_string(false), "'baz' command is not defined");
    }

    #[test]
    fn missing_command_error() {
        let mut args = Args::new(["noargs"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        cmd("foo").take(&mut args);
        cmd("bar").take(&mut args);
        let e = args.finish().expect_err("error");
        assert_eq!(e.to_string(false), "command is not specified");
    }

    #[test]
    fn parse_arg_error() {
        let mut args = Args::new(["noargs", "foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = arg("INTEGER")
            .take(&mut args)
            .parse::<usize>()
            .expect_err("error");
        assert_eq!(
            e.to_string(false),
            r#"argument 'INTEGER' has an invalid value "foo": invalid digit found in string"#
        );
    }

    #[test]
    fn parse_opt_error() {
        let mut args = Args::new(["noargs", "-f=bar"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = opt("foo")
            .short('f')
            .take(&mut args)
            .parse::<usize>()
            .expect_err("error");
        assert_eq!(
            e.to_string(false),
            r#"argument '-f' has an invalid value "bar": invalid digit found in string"#
        );
    }
}
