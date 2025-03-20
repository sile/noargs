use std::str::FromStr;

#[expect(dead_code)]
#[derive(Debug, Clone)]
enum LogEntry {
    Arg(ArgSpec),
    Flag(FlagSpec),
    Subcommand(SubcommandSpec),
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
        let mut raw_args = raw_args.map(Some).collect::<Vec<_>>();
        if !raw_args.is_empty() {
            raw_args[0] = None;
        }
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

        let after_index = spec.after_index.unwrap_or(0);
        let before_index = spec.before_index.unwrap_or(self.raw_args.len());

        if !spec.is_named() {
            let value = self.raw_args[after_index..before_index]
                .iter_mut()
                .find_map(|raw_arg| raw_arg.take());
            return Arg::new(spec, None, value);
        }

        for i in 0..self.raw_args.len() {
            if !(after_index..before_index).contains(&i) {
                continue;
            }
            let Some(raw_arg) = self.raw_args[i].take_if(|a| spec.name_matches(a)) else {
                continue;
            };

            if let Some((name, value)) = raw_arg.split_once('=') {
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

        let after_index = spec.after_index.unwrap_or(0);
        let before_index = spec.before_index.unwrap_or(self.raw_args.len());
        for (arg_index, raw_arg) in self.raw_args.iter_mut().enumerate() {
            if !(after_index..before_index).contains(&arg_index) {
                continue;
            }
            let Some(maybe_flag) = raw_arg else {
                continue;
            };

            if maybe_flag.starts_with("--") {
                if Some(&maybe_flag[2..]) == spec.long_name {
                    *raw_arg = None;
                    return Flag::new(spec, Some(FlagKind::LongName), Some(arg_index));
                }
            } else if let Some(name) = spec.short_name.filter(|_| maybe_flag.starts_with('-')) {
                if let Some((i, _)) = maybe_flag[1..].char_indices().find(|(_, c)| *c == name) {
                    maybe_flag.remove(i + 1);
                    if maybe_flag.len() == 1 {
                        *raw_arg = None;
                    }
                    return Flag::new(spec, Some(FlagKind::ShortName), Some(arg_index));
                }
            }
        }
        Flag::new(spec, None, None)
    }

    pub fn take_subcommand(&mut self, spec: SubcommandSpec) -> Subcommand {
        self.log.entries.push(LogEntry::Subcommand(spec));

        todo!()
    }

    // TODO: pub fn set_options_end() // for '--'

    // TODO: rename (e.g., check_unexpected_args())
    pub fn finish(self) -> Result<(), FinishError> {
        todo!()
    }

    // TODO: rename
    pub fn subcommand_error(self) {}
}

#[derive(Debug)]
pub enum SubcommandError {
    UnexpectedArgsBeforeSubcommand,
    UnexpectedSubcommandName,
    SubcommandNotFound,
}

impl SubcommandError {
    pub fn guess(_args: &Args) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum FinishError {
    UnexpectedArg,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<E> {
    InvalidValue {
        spec: ArgSpec,
        kind: ArgKind,
        error: E, // TODO: reason: String
    },
    NotFound {
        spec: ArgSpec,
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
            .and_then(|v| v.ok_or_else(|| ParseError::NotFound { spec: self.spec }))
    }

    pub fn parse_if_present<T: FromStr>(&self) -> Result<Option<T>, ParseError<T::Err>> {
        let Some(value) = self.value() else {
            if self.kind.is_some() {
                return Err(ParseError::MissingValue { spec: self.spec });
            } else {
                return Ok(None);
            }
        };
        value
            .parse()
            .map_err(|error| ParseError::InvalidValue {
                spec: self.spec,
                kind: self.kind.expect("infallible"),
                error,
            })
            .map(Some)
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
pub struct PositionalArgSpec {
    value_name: &'static str,
    doc: &'static str,
    default_value: Option<&'static str>,
    example_value: Option<&'static str>,
    before_index: Option<usize>,
    after_index: Option<usize>,
}

// TODO: Arg -> PositionalArg and NamedArg
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
    before_index: Option<usize>,
    after_index: Option<usize>,
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
            before_index: None,
            after_index: None,
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

    pub const fn example_if(mut self, condition: bool, example_value: &'static str) -> Self {
        if condition {
            self.example_value = Some(example_value);
        }
        self
    }

    pub const fn before(mut self, index: Option<usize>) -> Self {
        self.before_index = index;
        self
    }

    pub const fn after(mut self, index: Option<usize>) -> Self {
        self.after_index = index;
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
    index: Option<usize>,
}

impl Flag {
    fn new(spec: FlagSpec, kind: Option<FlagKind>, index: Option<usize>) -> Self {
        let kind = kind.or_else(|| {
            if spec.is_env_set() {
                Some(FlagKind::EnvVar)
            } else {
                None
            }
        });
        Self { spec, kind, index }
    }

    pub fn is_present(self) -> bool {
        self.kind.is_some()
    }

    pub fn kind(self) -> Option<FlagKind> {
        self.kind
    }

    pub fn index(self) -> Option<usize> {
        self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: &'static str,
    env: Option<&'static str>,
    before_index: Option<usize>,
    after_index: Option<usize>,
}

impl FlagSpec {
    pub const HELP: Self = Self::new().long("help").short('h');
    pub const VERSION: Self = Self::new().long("version");
    pub const OPTIONS_END: Self = Self::new().long(""); // TODO: rename

    pub const fn new() -> Self {
        Self {
            long_name: None,
            short_name: None,
            doc: "",
            env: None,
            before_index: None,
            after_index: None,
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

    pub const fn before(mut self, index: Option<usize>) -> Self {
        self.before_index = index;
        self
    }

    pub const fn after(mut self, index: Option<usize>) -> Self {
        self.after_index = index;
        self
    }

    fn is_env_set(self) -> bool {
        self.env
            .is_some_and(|name| std::env::var(name).is_ok_and(|v| !v.is_empty()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Subcommand {
    // TODO: index
    is_present: bool,
}

impl Subcommand {
    pub fn is_present(self) -> bool {
        self.is_present
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubcommandSpec {
    name: &'static str,
    doc: Option<&'static str>,
    // before / after
}

impl SubcommandSpec {
    pub const fn new(name: &'static str) -> Self {
        Self { name, doc: None }
    }

    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = Some(doc);
        self
    }
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
        assert_eq!(args.take_flag(flag).index(), Some(2));
        assert_eq!(args.take_flag(flag).index(), Some(4));
        assert!(!args.take_flag(flag).is_present());

        assert_eq!(args.remaining_raw_args().collect::<Vec<_>>(), ["run"]);

        // TODO: split test case
        let mut args = Args::new(raw_args(&["test", "--foo=1"]));

        let flag = FlagSpec::new().long("foo").env("TEST_TAKE_FLAG_FOO");
        assert!(!args.take_flag(flag).is_present());

        std::env::set_var("TEST_TAKE_FLAG_FOO", "1");
        assert!(args.take_flag(flag).is_present());

        // TODO: split test case
        let mut args = Args::new(raw_args(&["test", "-abc"]));

        let flag = FlagSpec::new().short('b');
        assert!(args.take_flag(flag).is_present());
        assert!(!args.take_flag(flag).is_present());

        let flag = FlagSpec::new().short('a');
        assert!(args.take_flag(flag).is_present());

        //
        let mut args = Args::new(raw_args(&["test", "--foo", "--", "--bar"]));
        let index = args.take_flag(FlagSpec::OPTIONS_END).index();

        let flag = FlagSpec::new().long("foo").before(index);
        assert!(args.take_flag(flag).is_present());

        let flag = FlagSpec::new().long("bar").before(index);
        assert!(!args.take_flag(flag).is_present());
    }

    #[test]
    fn take_arg() {
        let mut args = Args::new(raw_args(&["test", "--foo=1", "bar", "-b", "2", "qux"]));

        let arg = ArgSpec::new().short('b');
        assert_eq!(args.take_arg(arg).parse(), Ok(2));

        let arg = ArgSpec::new().long("foo");
        assert_eq!(args.take_arg(arg).parse(), Ok(1));

        let arg = ArgSpec::new().long("bar").default("3");
        assert_eq!(args.take_arg(arg).parse(), Ok(3));

        let arg = ArgSpec::new();
        assert_eq!(args.take_arg(arg).parse(), Ok("bar".to_owned()));

        let arg = ArgSpec::new();
        assert_eq!(args.take_arg(arg).parse(), Ok("qux".to_owned()));

        assert_eq!(args.remaining_raw_args().count(), 0);

        // TODO: error cases
    }

    fn raw_args(args: &'static [&str]) -> impl 'static + Iterator<Item = String> {
        args.iter().map(|&a| a.to_owned())
    }
}
