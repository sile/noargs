use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum LogEntry {
    Arg(Arg),
    Flag(Flag),
    Subcommand(Subcommand),
}

#[derive(Debug)]
struct Log {
    entries: Vec<LogEntry>,
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct HelpBuilder<'a> {
    log: &'a Log,
    app_name: &'static str,
    app_description: &'static str,
    for_terminal: bool,
}

impl<'a> HelpBuilder<'a> {
    pub fn build(self) -> String {
        todo!()
    }
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct Args {
    raw_args: Vec<Option<String>>,
    log: Log,
}

impl Args {
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

    pub fn finish(self) -> Result<(), FinishError> {
        todo!()
    }

    pub fn log(&self) -> &Log {
        &self.log
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
