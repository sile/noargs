use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum LogEntry {
    Arg(ArgSpec),
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

    pub fn take_arg(&mut self, spec: ArgSpec) -> Arg {
        self.log.entries.push(LogEntry::Arg(spec));

        if !spec.is_named() {
            let value = self.raw_args.iter_mut().find_map(|raw_arg| raw_arg.take());
            return Arg::new(spec, None, value);
        }

        for i in 0..self.raw_args.len() {
            let Some(raw_arg) = self.raw_args[i].take_if(|a| spec.name_matches(a)) else {
                continue;
            };

            let mut tokens = raw_arg.splitn(2, '=');
            if let (Some(name), Some(value)) = (tokens.next(), tokens.next()) {
                return Arg::new(spec, Some(name), Some(value.to_owned()));
            } else if let Some(value) = self.raw_args.get_mut(i + 1).and_then(|a| a.take()) {
                return Arg::new(spec, Some(&raw_arg), Some(value));
            } else {
                break;
            }
        }

        Arg::new(spec, None, None)
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
pub enum ParseError<E> {
    InvalidValue {
        spec: ArgSpec,
        kind: ArgKind,
        error: E,
    },
    MissingValue {
        spec: ArgSpec,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgKind {
    Positional,
    LongName,
    ShortName,
    EnvVar,
    Default,
    Example,
}

#[derive(Debug, Clone)]
pub struct Arg {
    spec: ArgSpec,
    kind: Option<ArgKind>,
    value: Option<String>,
}

impl Arg {
    pub fn new(spec: ArgSpec, name: Option<&str>, mut value: Option<String>) -> Self {
        let kind = if value.is_some() {
            if let Some(name) = name {
                if name.starts_with("--") {
                    Some(ArgKind::LongName)
                } else {
                    Some(ArgKind::ShortName)
                }
            } else {
                Some(ArgKind::Positional)
            }
        } else if let Some(v) = spec.env.and_then(|name| std::env::var(name).ok()) {
            value = Some(v);
            Some(ArgKind::EnvVar)
        } else if spec.default_value.is_some() {
            Some(ArgKind::Default)
        } else if spec.example_value.is_some() {
            Some(ArgKind::Example)
        } else {
            None
        };

        Self { spec, kind, value }
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, ParseError<T::Err>> {
        self.parse_if_present()
            .and_then(|v| v.ok_or_else(|| ParseError::MissingValue { spec: self.spec }))
    }

    pub fn parse_if_present<T: FromStr>(&self) -> Result<Option<T>, ParseError<T::Err>> {
        self.value()
            .map(|v| {
                v.parse().map_err(|error| ParseError::InvalidValue {
                    spec: self.spec,
                    kind: self.kind.expect("infallible"),
                    error,
                })
            })
            .transpose()
    }

    pub fn is_present(&self) -> bool {
        self.kind().is_some()
    }

    pub fn kind(&self) -> Option<ArgKind> {
        self.kind
    }

    pub fn value(&self) -> Option<&str> {
        match self.kind? {
            ArgKind::Positional | ArgKind::LongName | ArgKind::ShortName | ArgKind::EnvVar => {
                self.value.as_ref().map(|a| a.as_str())
            }
            ArgKind::Default => self.spec.default_value,
            ArgKind::Example => self.spec.example_value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: &'static str,
    doc: &'static str,
    env: Option<&'static str>,
    sensitive: bool,
    default_value: Option<&'static str>,
    example_value: Option<&'static str>,
}

impl ArgSpec {
    fn is_named(self) -> bool {
        self.long_name.is_some() || self.short_name.is_some()
    }

    fn name_matches(self, raw_arg: &str) -> bool {
        if raw_arg.starts_with("--") {
            let name = raw_arg[2..].splitn(2, '=').next().expect("infallible");
            Some(name) == self.long_name
        } else if raw_arg.starts_with('-') {
            let name = raw_arg[1..].splitn(2, '=').next().expect("infallible");
            let mut chars = name.chars();
            (chars.next(), chars.next()) == (self.short_name, None)
        } else {
            false
        }
    }

    pub const fn new() -> Self {
        Self {
            long_name: None,
            short_name: None,
            value_name: "VALUE",
            doc: "",
            env: None,
            sensitive: false,
            default_value: None,
            example_value: None,
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

    pub const fn value_name(mut self, name: &'static str) -> Self {
        self.value_name = name;
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

    pub const fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    pub const fn default(mut self, default_value: &'static str) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub const fn example(mut self, example_value: &'static str) -> Self {
        self.example_value = Some(example_value);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagKind {
    LongName,
    ShortName,
    EnvVar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flag {
    spec: FlagSpec,
    kind: Option<FlagKind>,
}

impl Flag {
    fn new(spec: FlagSpec, raw_arg: Option<String>) -> Self {
        let kind = if let Some(raw_arg) = raw_arg {
            if raw_arg.starts_with("--") {
                Some(FlagKind::LongName)
            } else {
                Some(FlagKind::ShortName)
            }
        } else if spec.is_env_set() {
            Some(FlagKind::EnvVar)
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
