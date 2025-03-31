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

    /// Minimum index that [`Flag::index()`] can have.
    pub min_index: Option<usize>,

    /// Maximum index that [`Flag::index()`] can have.
    pub max_index: Option<usize>,
}

impl FlagSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "",
        short: None,
        doc: "",
        env: None,
        min_index: None,
        max_index: None,
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

    /// Updates the value of [`FlagSpec::min_index`].
    pub const fn min_index(mut self, index: Option<usize>) -> Self {
        self.min_index = index;
        self
    }

    /// Updates the value of [`FlagSpec::max_index`].
    pub const fn max_index(mut self, index: Option<usize>) -> Self {
        self.max_index = index;
        self
    }

    /// Takes the first [`Flag`] instance that satisfies this specification from the raw arguments.
    pub fn take(self, args: &mut RawArgs) -> Flag {
        args.with_record_flag(|args| {
            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
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
        let mut args = args(&["test", "--foo"]);
        let flag = flag("foo");
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn short_name_flag() {
        let mut args = args(&["test", "-f", "-bf"]);

        let flag = short_flag('f');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        let flag = short_flag('b');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn env_flag() {
        let mut args = args(&["test", "--bar"]);

        let flag = FlagSpec {
            name: "foo",
            env: Some("TEST_ENV_FLAG_FOO"),
            ..Default::default()
        };
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        unsafe {
            std::env::set_var("TEST_ENV_FLAG_FOO", "1");
        }
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
    }

    #[test]
    fn flag_range() {
        let mut args = args(&["test", "--foo", "--foo", "--foo"]);

        let mut flag = flag("foo");
        flag.min_index = Some(2);
        flag.max_index = Some(2);
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        flag.max_index = None;
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 3, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    fn args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn flag(long_name: &'static str) -> FlagSpec {
        FlagSpec {
            name: long_name,
            ..Default::default()
        }
    }

    fn short_flag(short_name: char) -> FlagSpec {
        FlagSpec {
            name: "dummy",
            short: Some(short_name),
            ..Default::default()
        }
    }
}
