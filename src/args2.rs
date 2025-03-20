use std::str::FromStr;

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
}

impl CliArgs {
    pub fn take_arg<T: FromStr>(&mut self, spec: Arg) -> Result<T, TakeArgError<T::Err>> {
        todo!()
    }

    pub fn take_optional_arg<T: FromStr>(
        &mut self,
        spec: Arg,
    ) -> Result<Option<T>, TakeArgError<T::Err>> {
        todo!()
    }

    pub fn take_flag(&mut self, spec: Flag) -> bool {
        todo!()
    }

    pub fn take_subcommand(&mut self, spec: Subcommand) -> bool {
        todo!()
    }

    pub fn build_help_text(&self, _for_terminal: bool) -> String {
        todo!()
    }

    pub fn finish(self) -> Result<(), FinishError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum FinishError {
    UnknownArgs,
    UnknownSubcommand,
}

#[derive(Debug)]
pub enum TakeArgError<E> {
    ParseError {
        arg_name: ArgName,
        arg_value: String,
        error: E,
    },
    MissingValue {
        arg_name: ArgName,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgName {
    Long(&'static str),
    Short(char),
    Value(&'static str),
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Arg {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: Option<&'static str>, // TODO: remove Option
    doc: Option<&'static str>,        // TODO: remove Option
    env: Option<&'static str>,
    hidden_env: Option<&'static str>,
    default_value: Option<&'static str>,
}

impl Arg {
    pub fn representive_name(self) -> ArgName {
        todo!()
    }

    pub fn default_if(mut self, cond: bool, default: &'static str) -> Self {
        if !cond {
            return self;
        }

        self.default_value = Some(default);
        self
    }
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
