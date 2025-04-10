use crate::{
    args::{Metadata, RawArgs},
    error::Error,
};

/// Specification for [`Arg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgSpec {
    /// Value name (usually SCREAMING_SNAKE_CASE).
    pub name: &'static str,

    /// Documentation.
    pub doc: &'static str,

    /// Default value.
    pub default: Option<&'static str>,

    /// Example value (if this is set, the argument is considered to be requried when generating the help text).
    ///
    /// This is only used if `RawArgs::metadata().help_mode` is `true`.
    pub example: Option<&'static str>,

    /// Minimum index that [`Arg::index()`] can have.
    pub min_index: Option<usize>,

    /// Maximum index that [`Arg::index()`] can have.
    pub max_index: Option<usize>,
}

impl ArgSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "ARGUMENT",
        doc: "",
        default: None,
        example: None,
        min_index: None,
        max_index: None,
    };

    /// Makes an [`ArgSpec`] instance with a specified name (equivalent to `noargs::arg(name)`).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            ..Self::DEFAULT
        }
    }

    /// Updates the value of [`ArgSpec::doc`].
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }

    /// Updates the value of [`ArgSpec::default`].
    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    /// Updates the value of [`ArgSpec::example`].
    pub const fn example(mut self, example: &'static str) -> Self {
        self.example = Some(example);
        self
    }

    /// Updates the value of [`ArgSpec::min_index`].
    pub const fn min_index(mut self, index: Option<usize>) -> Self {
        self.min_index = index;
        self
    }

    /// Updates the value of [`ArgSpec::max_index`].
    pub const fn max_index(mut self, index: Option<usize>) -> Self {
        self.max_index = index;
        self
    }

    /// Takes the first [`Arg`] instance that satisfies this specification from the raw arguments.
    pub fn take(self, args: &mut RawArgs) -> Arg {
        let metadata = args.metadata();
        args.with_record_arg(|args| {
            if args.metadata().help_mode {
                return if self.default.is_some() {
                    Arg::Default {
                        spec: self,
                        metadata,
                    }
                } else if self.example.is_some() {
                    Arg::Example {
                        spec: self,
                        metadata,
                    }
                } else {
                    Arg::None { spec: self }
                };
            }

            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                if let Some(value) = raw_arg.value.take() {
                    return Arg::Positional {
                        spec: self,
                        metadata,
                        index,
                        value,
                    };
                };
            }

            if self.default.is_some() {
                Arg::Default {
                    spec: self,
                    metadata,
                }
            } else {
                Arg::None { spec: self }
            }
        })
    }
}

impl Default for ArgSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A positional argument.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Arg {
    Positional {
        spec: ArgSpec,
        metadata: Metadata,
        index: usize,
        value: String,
    },
    Default {
        spec: ArgSpec,
        metadata: Metadata,
    },
    Example {
        spec: ArgSpec,
        metadata: Metadata,
    },
    None {
        spec: ArgSpec,
    },
}

impl Arg {
    /// Parse the value of this argument.
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let value = self
            .is_present()
            .then_some(self.value())
            .ok_or_else(|| Error::MissingArg {
                arg: Box::new(self.clone()),
            })?;
        self.parse_with(|_| value.parse())
    }

    /// Parse the value of this argument if it is present.
    pub fn parse_if_present<T>(&self) -> Result<Option<T>, Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.is_present().then(|| self.parse()).transpose()
    }

    /// Similar to [`Arg::parse()`], but more flexible as this method allows you to specify an arbitrary parsing function.
    pub fn parse_with<F, T, E>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        f(self).map_err(|e| Error::ParseArgError {
            arg: Box::new(self.clone()),
            reason: e.to_string(),
        })
    }

    /// Returns the specification of this argument.
    pub fn spec(&self) -> ArgSpec {
        match self {
            Arg::Positional { spec, .. }
            | Arg::Default { spec, .. }
            | Arg::Example { spec, .. }
            | Arg::None { spec } => *spec,
        }
    }

    /// Returns `true` if this argument has a value.
    pub fn is_present(&self) -> bool {
        !matches!(self, Self::None { .. })
    }

    /// Returns the raw value of this argument.
    #[deprecated(since = "0.3.0", note = "please use `present()` and `value()` instead")]
    pub fn raw_value(&self) -> Option<&str> {
        self.is_present().then_some(self.value())
    }

    /// Returns the raw value of this argument, or an empty string if not present.
    #[deprecated(since = "0.3.0", note = "please use `value()` instead")]
    pub fn raw_value_or_empty(&self) -> &str {
        self.value()
    }

    /// Returns the raw value of this argument, or an empty string if not present.
    pub fn value(&self) -> &str {
        match self {
            Arg::Positional { value, .. } => value.as_str(),
            Arg::Default { spec, .. } => spec.default.unwrap_or(""),
            Arg::Example { spec, .. } => spec.example.unwrap_or(""),
            Arg::None { .. } => "",
        }
    }

    /// Returns the index at which the raw value of this argument was located in [`RawArgs`].
    pub fn index(&self) -> Option<usize> {
        if let Arg::Positional { index, .. } = self {
            Some(*index)
        } else {
            None
        }
    }

    pub(crate) fn metadata(&self) -> Option<Metadata> {
        match self {
            Arg::Positional { metadata, .. }
            | Arg::Default { metadata, .. }
            | Arg::Example { metadata, .. } => Some(*metadata),
            Arg::None { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_arg() {
        let mut args = args(&["test", "foo", "bar"]);
        let arg = arg("ARG");
        assert!(matches!(
            arg.take(&mut args),
            Arg::Positional { index: 1, .. }
        ));
        assert!(matches!(
            arg.take(&mut args),
            Arg::Positional { index: 2, .. }
        ));
        assert!(matches!(arg.take(&mut args), Arg::None { .. }));
    }

    #[test]
    fn optional_arg() {
        let mut args = args(&["test", "foo"]);
        let mut arg = arg("ARG");
        arg.default = Some("bar");
        assert!(matches!(
            arg.take(&mut args),
            Arg::Positional { index: 1, .. }
        ));
        assert!(matches!(arg.take(&mut args), Arg::Default { .. }));
        assert!(matches!(arg.take(&mut args), Arg::Default { .. }));
    }

    #[test]
    fn example_arg() {
        let mut args = args(&["test", "foo"]);
        args.metadata_mut().help_mode = true;

        let mut arg = arg("ARG");
        arg.example = Some("bar");
        assert!(matches!(arg.take(&mut args), Arg::Example { .. }));
        assert!(matches!(arg.take(&mut args), Arg::Example { .. }));
    }

    #[test]
    fn parse_arg() {
        let mut args = args(&["test", "1", "not a number"]);
        let arg = arg("ARG");
        assert_eq!(arg.take(&mut args).parse::<usize>().ok(), Some(1));
        assert_eq!(arg.take(&mut args).parse::<usize>().ok(), None);
        assert_eq!(arg.take(&mut args).parse::<usize>().ok(), None);
    }

    fn args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn arg(name: &'static str) -> ArgSpec {
        ArgSpec {
            name,
            ..Default::default()
        }
    }
}
