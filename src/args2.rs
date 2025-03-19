use std::str::FromStr;

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    positional_args_start: usize,
    named_args_end: usize,
}

impl CliArgs {}

pub trait TakeArg<SPEC> {
    type Value;

    fn take_arg(&mut self, spec: SPEC) -> Self::Value;
}

impl TakeArg<CliFlagSpec> for CliArgs {
    type Value = bool;

    fn take_arg(&mut self, _spec: CliFlagSpec) -> Self::Value {
        todo!()
    }
}

impl TakeArg<CliOptionSpec> for CliArgs {
    type Value = CliOptionValue;

    fn take_arg(&mut self, _spec: CliOptionSpec) -> Self::Value {
        todo!()
    }
}

#[derive(Debug)]
pub enum ParseError<SPEC, E> {
    InvalidValue {
        arg_spec: SPEC,
        arg_value: String,
        error: E,
    },
    MissingOptionValue {
        arg_spec: SPEC,
    },
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Arg {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: Option<&'static str>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
    hidden_env: Option<&'static str>,
    default_value: Option<&'static str>,
    example_value: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct OptionArg {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: Option<&'static str>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
    hidden_env: Option<&'static str>,
    default_value: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Flag {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Subcommand {
    name: &'static str,
    doc: Option<&'static str>,
}

// fn foo() {
//     args.take_arg(FOO.default_if(help, "foo")).parse()?;

//     if args.take_arg(COMMAND_RUN) {
//     } else if args.take_arg(COMMAND_FOO) {
//     } else if help {
//         println!("{}", args.help(std::io::stdio().is_terminal()));
//     } else {
//         args.finish()?;
//     }
// }

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct CliArgSpec {
    value_name: &'static str,
    example_value: Option<&'static str>,
    // TODO: env, hidden_env
    doc: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct CliOptionalArgSpec {
    value_name: &'static str,
    doc: Option<&'static str>,
}

#[derive(Debug)]
pub struct CliOptionValue {
    spec: CliOptionSpec,
    value: Option<String>,
    missing_value: bool,
}

impl CliOptionValue {
    // TODO: parse_and(self, f: FnOnce)

    pub fn parse<T: FromStr>(self) -> Result<Option<T>, ParseError<CliOptionSpec, T::Err>> {
        if self.missing_value {
            return Err(ParseError::MissingOptionValue {
                arg_spec: self.spec,
            });
        }

        let Some(value) = self.value else {
            return Ok(None);
        };

        value
            .parse()
            .map(Some)
            .map_err(|error| ParseError::InvalidValue {
                arg_spec: self.spec,
                arg_value: value,
                error,
            })
    }
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct CliOptionSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct CliOptionWithDefaultSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    default_value: &'static str,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliRequiredOptionSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    example_value: Option<&'static str>,
    value: String,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct CliFlagSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
}

impl CliFlagSpec {
    pub const HELP: Self = Self::new("help", 'h').doc("Print help");
    pub const VERSION: Self = Self::long("version").doc("Print version");
    pub const OPTIONS_END: Self = Self::long("").doc("Indicate options end");

    pub const fn new(long_name: &'static str, short_name: char) -> Self {
        Self {
            long_name: Some(long_name),
            short_name: Some(short_name),
            doc: None,
        }
    }

    pub const fn long(name: &'static str) -> Self {
        Self {
            long_name: Some(name),
            short_name: None,
            doc: None,
        }
    }

    pub const fn short(name: char) -> Self {
        Self {
            long_name: None,
            short_name: Some(name),
            doc: None,
        }
    }

    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = Some(doc);
        self
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
