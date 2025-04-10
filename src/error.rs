use std::io::IsTerminal;

use crate::{Arg, Metadata, Opt, OptSpec, RawArgs, args::Taken, formatter::Formatter};

/// Possible errors.
///
/// Note that this enum is intended to be used only as a top-level error and
/// deliberately does not implement the [`std::error::Error`] and [`std::fmt::Display`] traits.
///
/// Additionally, any external errors that implement [`std::fmt::Display`] can be converted into this error.
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Error {
    UnexpectedArg {
        metadata: Metadata,
        raw_arg: String,
    },
    UndefinedCommand {
        metadata: Metadata,
        raw_arg: String,
    },
    MissingCommand {
        metadata: Metadata,
    },
    InvalidArg {
        arg: Box<Arg>,
        reason: String,
    },
    MissingArg {
        arg: Box<Arg>,
    },
    InvalidOpt {
        opt: Box<Opt>,
        reason: String,
    },
    MissingOpt {
        opt: Box<Opt>,
    },
    Other {
        metadata: Option<Metadata>,
        error: Box<dyn std::fmt::Display>,
    },
}

impl Error {
    /// Makes an application specific error.
    pub fn other<E>(args: &RawArgs, error: E) -> Self
    where
        E: 'static + std::fmt::Display,
    {
        Self::Other {
            metadata: Some(args.metadata()),
            error: Box::new(error),
        }
    }

    pub(crate) fn check_command_error(args: &RawArgs) -> Result<(), Error> {
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

    pub(crate) fn check_unexpected_arg(args: &RawArgs) -> Result<(), Error> {
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
            Error::InvalidArg { arg, reason } => {
                fmt.write(&format!(
                    "argument '{}' has an invalid value {:?}: {reason}",
                    fmt.bold(arg.spec().name),
                    arg.value()
                ));
                if let Some(metadata) = arg.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            Error::MissingArg { arg } => {
                fmt.write(&format!("missing argument '{}'", fmt.bold(arg.spec().name)));
                if let Some(metadata) = arg.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            Error::InvalidOpt { opt, reason } => {
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
                    _ => format!("argument '{}'", fmt.bold(&format!("--{}", opt.spec().name))),
                };
                fmt.write(&format!(
                    "{name} has an invalid value {:?}: {reason}",
                    opt.value()
                ));
                if let Some(metadata) = opt.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            Error::MissingOpt { opt } => {
                match **opt {
                    Opt::MissingValue {
                        spec:
                            OptSpec {
                                short: Some(name), ..
                            },
                        long: false,
                    } => {
                        let name = fmt.bold(&format!("-{name}")).into_owned();
                        fmt.write(&format!("missing '{name}' value"));
                    }
                    Opt::MissingValue { spec, .. } => {
                        let name = fmt.bold(&format!("--{}", spec.name)).into_owned();
                        fmt.write(&format!("missing '{name}' value"));
                    }
                    _ => {
                        let name = fmt.bold(&format!("--{}", opt.spec().name)).into_owned();
                        fmt.write(&format!("missing '{name}' option"));
                    }
                };
                if let Some(metadata) = opt.metadata() {
                    metadata
                } else {
                    return fmt.finish();
                }
            }
            Error::Other {
                metadata: Some(metadata),
                error,
            } => {
                fmt.write(&error.to_string());
                *metadata
            }
            Error::Other {
                metadata: None,
                error,
            } => {
                fmt.write(&error.to_string());
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
        Self::Other {
            metadata: None,
            error: Box::new(error),
        }
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
        let args = RawArgs::new(["noargs"].iter().map(|a| a.to_string()));
        assert!(Error::check_unexpected_arg(&args).is_ok());

        // Error without `--help`.
        let mut args = RawArgs::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(e.to_string(false), "unexpected argument '--foo' found");

        // Error with `--help`.
        let args = RawArgs::new(["noargs", "--foo"].iter().map(|a| a.to_string()));
        let e = Error::check_unexpected_arg(&args).expect_err("should error");
        assert_eq!(
            e.to_string(false),
            r#"unexpected argument '--foo' found

Try '--help' for more information."#
        );
    }

    #[test]
    fn undefined_command_error() {
        let mut args = RawArgs::new(["noargs", "baz"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        cmd("foo").take(&mut args);
        cmd("bar").take(&mut args);
        let e = args.finish().expect_err("error");
        assert_eq!(e.to_string(false), "'baz' command is not defined");
    }

    #[test]
    fn missing_command_error() {
        let mut args = RawArgs::new(["noargs"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        cmd("foo").take(&mut args);
        cmd("bar").take(&mut args);
        let e = args.finish().expect_err("error");
        assert_eq!(e.to_string(false), "command is not specified");
    }

    #[test]
    fn parse_arg_error() {
        let mut args = RawArgs::new(["noargs", "foo"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = arg("INTEGER")
            .take(&mut args)
            .then(|a| a.value().parse::<usize>())
            .expect_err("error");
        assert_eq!(
            e.to_string(false),
            r#"argument 'INTEGER' has an invalid value "foo": invalid digit found in string"#
        );
    }

    #[test]
    fn parse_opt_error() {
        let mut args = RawArgs::new(["noargs", "-f=bar"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = opt("foo")
            .short('f')
            .take(&mut args)
            .then(|o| o.value().parse::<usize>())
            .expect_err("error");
        assert_eq!(
            e.to_string(false),
            r#"argument '-f' has an invalid value "bar": invalid digit found in string"#
        );
    }

    #[test]
    fn missing_arg_error() {
        let mut args = RawArgs::new(["noargs"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = arg("INTEGER")
            .take(&mut args)
            .then(|a| a.value().parse::<usize>())
            .expect_err("error");
        assert_eq!(e.to_string(false), "missing argument 'INTEGER'");
    }

    #[test]
    fn missing_opt_error() {
        let mut args = RawArgs::new(["noargs", "-f"].iter().map(|a| a.to_string()));
        args.metadata_mut().help_flag_name = None;
        let e = opt("foo")
            .short('f')
            .take(&mut args)
            .then(|o| o.value().parse::<usize>())
            .expect_err("error");
        assert_eq!(e.to_string(false), "missing '-f' value");
    }
}
