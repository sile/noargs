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

impl Log {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
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
pub struct Args {
    raw_args: Vec<Option<String>>,
    log: Log,
}

impl Args {
    pub fn new<I>(raw_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let raw_args = raw_args.skip(1).map(Some).collect();
        Self {
            raw_args,
            log: Log::new(),
        }
    }

    pub fn remaining_raw_args(&self) -> impl Iterator<Item = &str> {
        self.raw_args
            .iter()
            .filter_map(|a| a.as_ref().map(|a| a.as_str()))
    }

    pub fn take_arg<T: FromStr>(&mut self, spec: Arg) -> Result<T, TakeArgError<T::Err>> {
        todo!()
    }

    pub fn take_optional_arg<T: FromStr>(
        &mut self,
        spec: Arg,
    ) -> Result<Option<T>, TakeArgError<T::Err>> {
        todo!()
    }

    pub fn take_flag(&mut self, flag: Flag) -> bool {
        self.log.entries.push(LogEntry::Flag(flag));

        self.raw_args
            .iter_mut()
            .find(|raw_arg| {
                let Some(raw_arg) = raw_arg else {
                    return false;
                };
                name_matches(raw_arg, flag.long_name, flag.short_name)
            })
            .map(|raw_arg| raw_arg.take())
            .is_some()
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

fn name_matches(raw_arg: &str, long_name: Option<&str>, short_name: Option<char>) -> bool {
    if raw_arg.starts_with("--") {
        Some(&raw_arg[2..]) == long_name
    } else if raw_arg.starts_with('-') {
        let mut chars = raw_arg[1..].chars();
        (chars.next(), chars.next()) == (short_name, None)
    } else {
        false
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code)]
pub struct Flag {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: &'static str,
    env: Option<&'static str>,
}

impl Flag {
    pub const HELP: Self = Self::new().long("help").short('h');
    pub const VERSION: Self = Self::new().long("version");
    pub const OPTIONS_END: Self = Self::new().long("");

    pub const fn new() -> Self {
        Self {
            long_name: None,
            short_name: None,
            doc: "",
            env: None,
        }
    }

    pub const fn long(mut self, name: &'static str) -> Self {
        self.long_name = Some(name);
        self
    }

    pub const fn short(mut self, name: char) -> Self {
        self.short_name = Some(name);
        self
    }

    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }

    pub const fn env(mut self, variable_name: &'static str) -> Self {
        self.env = Some(variable_name);
        self
    }
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Subcommand {
    name: &'static str,
    doc: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_new() {
        // The first raw argument is regarded as the command name and will be ignored.
        let args = Args::new(raw_args(&["test", "run"]));
        assert_eq!(args.remaining_raw_args().count(), 1);

        // `Args::new()` does not panic even if raw arguments are empty.
        let args = Args::new(raw_args(&[]));
        assert_eq!(args.remaining_raw_args().count(), 0);
    }

    #[test]
    fn take_flag() {
        let mut args = Args::new(raw_args(&["test", "--foo", "--bar", "run", "-b"]));

        let flag = Flag::new().long("foo");
        assert!(args.take_flag(flag));
        assert!(!args.take_flag(flag));

        let flag = Flag::new().long("bar").short('b');
        assert!(args.take_flag(flag));
        assert!(args.take_flag(flag));
        assert!(!args.take_flag(flag));

        assert_eq!(args.remaining_raw_args().collect::<Vec<_>>(), ["run"]);
    }

    fn raw_args(args: &'static [&str]) -> impl 'static + Iterator<Item = String> {
        args.iter().map(|&a| a.to_owned())
    }
}
