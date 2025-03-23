use std::str::FromStr;

use crate::{
    args::{Args, Metadata},
    error::Error,
};

/// Specification for [`Arg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgSpec {
    /// Value name (usually SCREAMING_SNAKE_CASE).
    pub name: &'static str,

    /// Argument documentation.
    pub doc: &'static str,

    /// Default value.
    pub default: Option<&'static str>,

    /// Example value (if this is set, the argument is considered to be requried).
    ///
    /// This is only used if `Args::metadata().help_mode` is `true`.
    pub example: Option<&'static str>,

    /// Minimum index that [`Arg::index()`] can have.
    pub min_index: Option<usize>,

    /// Maximum index that [`Arg::index()`] can have.
    pub max_index: Option<usize>,

    metadata: Metadata,
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
        metadata: Metadata::DEFAULT,
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
    pub fn take(mut self, args: &mut Args) -> Arg {
        self.metadata = args.metadata();
        args.with_record_arg(|args| {
            if args.metadata().help_mode {
                return if self.default.is_some() {
                    Arg::Default { spec: self }
                } else if self.example.is_some() {
                    Arg::Example { spec: self }
                } else {
                    Arg::None { spec: self }
                };
            }

            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                if let Some(value) = raw_arg.value.take() {
                    return Arg::Positional {
                        spec: self,
                        index,
                        raw_value: value,
                    };
                };
            }

            if self.default.is_some() {
                Arg::Default { spec: self }
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
        index: usize,
        raw_value: String,
    },
    Default {
        spec: ArgSpec,
    },
    Example {
        spec: ArgSpec,
    },
    None {
        spec: ArgSpec,
    },
}

impl Arg {
    /// Parse the value of this argument.
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let value = self.raw_value().ok_or_else(|| Error::MissingArg {
            arg: Box::new(self.clone()),
        })?;
        value.parse::<T>().map_err(|e| Error::ParseArgError {
            arg: Box::new(self.clone()),
            reason: e.to_string(),
        })
    }

    /// Parse the value of this argument if it is present.
    pub fn parse_if_present<T>(&self) -> Result<Option<T>, Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.is_present().then(|| self.parse()).transpose()
    }

    /// Similar to [`Arg::parse()`], but more flexible as this method allows you to specify an arbitrary parsing function.
    pub fn parse_with<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Self) -> Result<T, Error>,
    {
        f(self)
    }

    /// Returns the specification of this argument.
    pub fn spec(&self) -> ArgSpec {
        match self {
            Arg::Positional { spec, .. }
            | Arg::Default { spec }
            | Arg::Example { spec }
            | Arg::None { spec } => *spec,
        }
    }

    /// Returns `true` if this argument has a value.
    pub fn is_present(&self) -> bool {
        !matches!(self, Self::None { .. })
    }

    /// Returns the raw value of this argument.
    pub fn raw_value(&self) -> Option<&str> {
        match self {
            Arg::Positional { raw_value, .. } => Some(raw_value.as_str()),
            Arg::Default { spec } => spec.default,
            Arg::Example { spec } => spec.example,
            Arg::None { .. } => None,
        }
    }

    /// Returns the index at which the raw value of this argument was located in [`Args`].
    pub fn index(&self) -> Option<usize> {
        if let Arg::Positional { index, .. } = self {
            Some(*index)
        } else {
            None
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

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn arg(name: &'static str) -> ArgSpec {
        ArgSpec {
            name,
            ..Default::default()
        }
    }
}
