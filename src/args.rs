use std::{borrow::Cow, io::IsTerminal};

use crate::{Arg, Cmd, Error, Flag, Opt, help::HelpBuilder};
#[expect(unused_imports)]
use crate::{ArgSpec, OptSpec};

/// Raw arguments that will be converted into [`Arg`], [`Opt`], [`Flag`] and [`Cmd`] instances.
#[derive(Debug)]
pub struct RawArgs {
    metadata: Metadata,
    raw_args: Vec<RawArg>,
    log: Vec<Taken>,
}

impl RawArgs {
    /// Makes an [`RawArgs`] instance with the given raw arguments.
    pub fn new<I>(args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let raw_args = args
            .enumerate()
            .map(|(i, value)| RawArg {
                value: (i != 0).then_some(value),
            })
            .collect();
        Self {
            metadata: Metadata::default(),
            raw_args,
            log: Vec::new(),
        }
    }

    /// Returns the metadata.
    pub fn metadata(&self) -> Metadata {
        self.metadata
    }

    /// Returns a mutable reference of the metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Returns an iterator that iterates over unconsumed (not taken) raw arguments and their indices.
    pub fn remaining_args(&self) -> impl '_ + Iterator<Item = (usize, &str)> {
        self.raw_args
            .iter()
            .enumerate()
            .filter_map(|(i, a)| a.value.as_ref().map(|v| (i, v.as_str())))
    }

    /// Completes the parsing process and checks for any errors.
    ///
    /// If successful and [`Metadata::help_mode`] is `true`, this method returns `Ok(Some(help_text))`.
    pub fn finish(self) -> Result<Option<String>, Error> {
        if self.metadata.help_mode {
            let help = HelpBuilder::new(&self, std::io::stdout().is_terminal()).build();
            Ok(Some(help))
        } else {
            Error::check_command_error(&self)?;
            Error::check_unexpected_arg(&self)?;
            Ok(None)
        }
    }

    pub(crate) fn raw_args_mut(&mut self) -> &mut [RawArg] {
        &mut self.raw_args
    }

    pub(crate) fn log(&self) -> &[Taken] {
        &self.log
    }

    pub(crate) fn with_record_arg<F>(&mut self, f: F) -> Arg
    where
        F: FnOnce(&mut Self) -> Arg,
    {
        let arg = f(self);
        self.log.push(Taken::Arg(arg.clone()));
        arg
    }

    pub(crate) fn with_record_opt<F>(&mut self, f: F) -> Opt
    where
        F: FnOnce(&mut Self) -> Opt,
    {
        let opt = f(self);
        self.log.push(Taken::Opt(opt.clone()));
        opt
    }

    pub(crate) fn with_record_flag<F>(&mut self, f: F) -> Flag
    where
        F: FnOnce(&mut Self) -> Flag,
    {
        let flag = f(self);
        self.log.push(Taken::Flag(flag));
        flag
    }

    pub(crate) fn with_record_cmd<F>(&mut self, f: F) -> Cmd
    where
        F: FnOnce(&mut Self) -> Cmd,
    {
        let cmd = f(self);
        self.log.push(Taken::Cmd(cmd));
        cmd
    }

    pub(crate) fn next_raw_arg_value(&self) -> Option<&str> {
        self.raw_args.iter().find_map(|a| a.value.as_deref())
    }
}

#[derive(Debug, Clone)]
pub struct RawArg {
    pub value: Option<String>,
}

/// Metadata of [`RawArgs`].
#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    /// Application name (e.g., `env!("CARGO_PKG_NAME")`).
    pub app_name: &'static str,

    /// Application description (e.g., `env!("CARGO_PKG_DESCRIPTION")`).
    pub app_description: &'static str,

    /// Flag name for help (default: `Some("help")`).
    pub help_flag_name: Option<&'static str>,

    /// When enabled, the following help mode behaviors apply:
    ///
    /// - [`RawArgs::finish()`] will return `Ok(Some(help_text))` if successful
    /// - Only default and example values will be used when calling [`ArgSpec::take()`] or [`OptSpec::take()`]
    pub help_mode: bool,

    /// If `true`, a full help text will be displayed.
    pub full_help: bool,

    /// Predicate function to determine if a string contains only valid flag characters.
    ///
    /// This function is used when parsing short flags to distinguish between:
    /// - Multiple flags (e.g., `-abc` where each character is a flag)
    /// - Options with concatenated values (e.g., `-khello` where 'k' is an option and "hello" is its value)
    ///
    /// The default implementation accepts only ASCII alphabetic characters, which prevents
    /// ambiguity in parsing. For example, with `-khello world`, the presence of space and
    /// non-alphabetic characters indicates this is an option with a concatenated value rather
    /// than multiple flags.
    ///
    /// # Example: Only accept flags actually defined by the app
    ///
    /// ```rust
    /// use noargs::{raw_args, flag};
    ///
    /// let mut args = raw_args();
    ///
    /// // Define the valid short flags for your app
    /// const VALID_FLAGS: &[char] = &['h', 'v', 'q', 'd'];
    ///
    /// // Only allow characters that correspond to actual flags in your app
    /// args.metadata_mut().is_valid_flag_chars = |chars| {
    ///     chars.chars().all(|c| VALID_FLAGS.contains(&c))
    /// };
    ///
    /// // Now only -h, -v, -q, -d and their combinations (like -hv, -vd) are valid
    /// // Anything else like -khello be treated as an option with concatenated value
    /// let help_flag = flag("help").short('h').take(&mut args);
    /// let verbose_flag = flag("verbose").short('v').take(&mut args);
    /// let quiet_flag = flag("quiet").short('q').take(&mut args);
    /// let debug_flag = flag("debug").short('d').take(&mut args);
    /// ```
    pub is_valid_flag_chars: fn(&str) -> bool,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            app_name: "<APP_NAME>",
            app_description: "",
            help_flag_name: Some("help"),
            help_mode: false,
            full_help: false,
            is_valid_flag_chars: |chars| chars.chars().all(|c| c.is_ascii_alphabetic()),
        }
    }
}

// [NOTE]
// PartialEq, Eq, Hash are manually implemented to avoid
// the `unpredictable_function_pointer_comparisons` warning.
// (`is_valid_flag_chars` should not be compared)
//
// TODO: Remove `is_valid_flag_chars` from `Metadata`

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.app_name == other.app_name
            && self.app_description == other.app_description
            && self.help_flag_name == other.help_flag_name
            && self.help_mode == other.help_mode
            && self.full_help == other.full_help
    }
}

impl Eq for Metadata {}

impl std::hash::Hash for Metadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.app_name.hash(state);
        self.app_description.hash(state);
        self.help_flag_name.hash(state);
        self.help_mode.hash(state);
        self.full_help.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Taken {
    Arg(Arg),
    Opt(Opt),
    Flag(Flag),
    Cmd(Cmd),
}

impl Taken {
    pub fn name(&self) -> &'static str {
        match self {
            Taken::Arg(arg) => arg.spec().name,
            Taken::Opt(opt) => opt.spec().name,
            Taken::Flag(flag) => flag.spec().name,
            Taken::Cmd(cmd) => cmd.spec().name,
        }
    }

    pub fn example(&self) -> Option<Cow<'static, str>> {
        match self {
            Taken::Arg(arg) => arg.spec().example.map(Self::quote_if_need),
            Taken::Opt(opt) => opt
                .spec()
                .example
                .map(|v| Cow::Owned(format!("--{} {}", opt.spec().name, Self::quote_if_need(v)))),
            Taken::Cmd(cmd) if cmd.is_present() => Some(Cow::Borrowed(cmd.spec().name)),
            _ => None,
        }
    }

    fn quote_if_need(s: &'static str) -> Cow<'static, str> {
        if s.contains('"') && !s.contains('\'') {
            Cow::Owned(format!("'{}'", s))
        } else if s.contains([' ', '\'']) {
            Cow::Owned(format!("{:?}", s))
        } else {
            Cow::Borrowed(s)
        }
    }
}
