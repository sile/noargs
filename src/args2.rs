use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum LogEntry {
    Arg(Arg),
    Flag(FlagSpec),
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

    pub fn take_flag(&mut self, spec: FlagSpec) -> Flag {
        self.log.entries.push(LogEntry::Flag(spec));

        let raw_arg = self
            .raw_args
            .iter_mut()
            .find_map(|raw_arg| raw_arg.take_if(|a| spec.matches(a)));
        Flag::new(spec, raw_arg)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagKind {
    Long,
    Short,
    Env,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flag {
    spec: FlagSpec,
    kind: Option<FlagKind>,
}

impl Flag {
    fn new(spec: FlagSpec, raw_arg: Option<String>) -> Self {
        let kind = if let Some(raw_arg) = raw_arg {
            if raw_arg.starts_with("--") && Some(&raw_arg[2..]) == spec.long_name {
                Some(FlagKind::Long)
            } else {
                Some(FlagKind::Short)
            }
        } else if spec.is_env_set() {
            Some(FlagKind::Env)
        } else {
            None
        };
        Self { spec, kind }
    }

    pub fn is_present(&self) -> bool {
        self.kind.is_some()
    }

    pub fn kind(&self) -> Option<FlagKind> {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: &'static str,
    env: Option<&'static str>,
}

impl FlagSpec {
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

    fn is_env_set(self) -> bool {
        self.env
            .is_some_and(|name| std::env::var(name).is_ok_and(|v| !v.is_empty()))
    }

    fn matches(self, raw_arg: &str) -> bool {
        if raw_arg.starts_with("--") {
            Some(&raw_arg[2..]) == self.long_name
        } else if raw_arg.starts_with('-') {
            let mut chars = raw_arg[1..].chars();
            (chars.next(), chars.next()) == (self.short_name, None)
        } else {
            false
        }
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

        let flag = FlagSpec::new().long("foo");
        assert!(args.take_flag(flag).is_present());
        assert!(!args.take_flag(flag).is_present());

        let flag = FlagSpec::new().long("bar").short('b');
        assert!(args.take_flag(flag).is_present());
        assert!(args.take_flag(flag).is_present());
        assert!(!args.take_flag(flag).is_present());

        assert_eq!(args.remaining_raw_args().collect::<Vec<_>>(), ["run"]);

        let mut args = Args::new(raw_args(&["test", "--foo=1"]));

        let flag = FlagSpec::new().long("foo").env("TEST_TAKE_FLAG_FOO");
        assert!(!args.take_flag(flag).is_present());

        std::env::set_var("TEST_TAKE_FLAG_FOO", "1");
        assert!(args.take_flag(flag).is_present());
    }

    // #[test]
    // fn take_arg() {
    //     let mut args = Args::new(raw_args(&["test", "--foo=1", "bar", "-b", "2", "qux"]));
    // }

    fn raw_args(args: &'static [&str]) -> impl 'static + Iterator<Item = String> {
        args.iter().map(|&a| a.to_owned())
    }
}
