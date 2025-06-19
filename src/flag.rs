use crate::args::RawArgs;

/// Specification for [`Flag`].
///
/// Note that `noargs` does not support flags with only short names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    /// Flag long name (usually kebab-case).
    pub name: &'static str,

    /// Flag short name.
    pub short: Option<char>,

    /// Documentation.
    pub doc: &'static str,

    /// Environment variable name.
    ///
    /// If a non-empty value is set to this variable, this flag is considered to be set.
    pub env: Option<&'static str>,
}

impl FlagSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "",
        short: None,
        doc: "",
        env: None,
    };

    /// Makes an [`FlagSpec`] instance with a specified name (equivalent to `noargs::flag(name)`).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            ..Self::DEFAULT
        }
    }

    /// Updates the value of [`FlagSpec::short`].
    pub const fn short(mut self, name: char) -> Self {
        self.short = Some(name);
        self
    }

    /// Updates the value of [`FlagSpec::doc`].
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }

    /// Updates the value of [`FlagSpec::env`].
    pub const fn env(mut self, variable_name: &'static str) -> Self {
        self.env = Some(variable_name);
        self
    }

    /// Takes the first [`Flag`] instance that satisfies this specification from the raw arguments.
    pub fn take(self, args: &mut RawArgs) -> Flag {
        let is_valid_flag_chars = args.metadata().is_valid_flag_chars;
        args.with_record_flag(|args| {
            for (index, raw_arg) in args.raw_args_mut().iter_mut().enumerate() {
                let Some(value) = &mut raw_arg.value else {
                    continue;
                };
                if !value.starts_with('-') {
                    continue;
                }

                if value.starts_with("--") {
                    if &value[2..] == self.name {
                        raw_arg.value = None;
                        return Flag::Long { spec: self, index };
                    }
                } else if !(is_valid_flag_chars)(&value[1..]) {
                } else if let Some(i) = value
                    .char_indices()
                    .skip(1)
                    .find_map(|(i, c)| (Some(c) == self.short).then_some(i))
                {
                    value.remove(i);
                    if value.len() == 1 {
                        raw_arg.value = None;
                    }
                    return Flag::Short { spec: self, index };
                }
            }

            if self
                .env
                .is_some_and(|name| std::env::var(name).is_ok_and(|v| !v.is_empty()))
            {
                Flag::Env { spec: self }
            } else {
                Flag::None { spec: self }
            }
        })
    }

    /// Similar to [`FlagSpec::take()`], but updates the help-related metadata of `args` when the flag is present.
    ///
    /// Specifically, the following code is executed:
    /// ```no_run
    /// # use noargs::Flag;
    /// # let mut args = noargs::raw_args();
    /// # let flag = noargs::HELP_FLAG.take_help(&mut args);
    /// args.metadata_mut().help_mode = true;
    /// args.metadata_mut().help_flag_name = Some(flag.spec().name);
    /// if matches!(flag, Flag::Long { .. }) {
    ///     args.metadata_mut().full_help = true;
    /// }
    /// ```
    pub fn take_help(self, args: &mut RawArgs) -> Flag {
        let flag = self.take(args);
        if flag.is_present() {
            args.metadata_mut().help_mode = true;
            args.metadata_mut().help_flag_name = Some(self.name);
            if matches!(flag, Flag::Long { .. }) {
                args.metadata_mut().full_help = true;
            }
        }
        flag
    }
}

impl Default for FlagSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A named argument without value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Flag {
    Long { spec: FlagSpec, index: usize },
    Short { spec: FlagSpec, index: usize },
    Env { spec: FlagSpec },
    None { spec: FlagSpec },
}

impl Flag {
    /// Returns the specification of this flag.
    pub fn spec(self) -> FlagSpec {
        match self {
            Flag::Short { spec, .. }
            | Flag::Long { spec, .. }
            | Flag::Env { spec }
            | Flag::None { spec } => spec,
        }
    }

    /// Returns `true` if this flag is set.
    pub fn is_present(self) -> bool {
        !matches!(self, Flag::None { .. })
    }

    /// Returns `Some(self)` if this flag is present.
    pub fn present(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    /// Returns the index at which the raw value associated with this flag was located in [`RawArgs`].
    pub fn index(self) -> Option<usize> {
        match self {
            Flag::Short { index, .. } | Flag::Long { index, .. } => Some(index),
            Flag::Env { .. } | Flag::None { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_name_flag() {
        let mut args = test_args(&["test", "--foo"]);
        let flag = crate::flag("foo");
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn short_name_flag() {
        let mut args = test_args(&["test", "-f", "-bf"]);

        let flag = crate::flag("dummy").short('f');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        let flag = crate::flag("dummy").short('b');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn env_flag() {
        let mut args = test_args(&["test", "--bar"]);

        let flag = crate::flag("foo").env("TEST_ENV_FLAG_FOO");
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        unsafe {
            std::env::set_var("TEST_ENV_FLAG_FOO", "1");
        }
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
    }

    fn test_args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }
}
