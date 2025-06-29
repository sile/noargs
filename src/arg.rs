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
}

impl ArgSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "<ARGUMENT>",
        doc: "",
        default: None,
        example: None,
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

            for (index, raw_arg) in args.raw_args_mut().iter_mut().enumerate() {
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

    /// Returns `Some(self)` if this argument is present.
    pub fn present(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    /// Applies additional conversion or validation to the argument.
    ///
    /// This method allows for chaining transformations and validations when an argument is present.
    /// It first checks if the argument has a value and then applies the provided function.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut args = noargs::RawArgs::new(["example", "42"].iter().map(|a| a.to_string()));
    /// let arg = noargs::arg("<NUMBER>").take(&mut args);
    ///
    /// // Parse as number and ensure it's positive
    /// let num = arg.then(|arg| -> Result<_, Box<dyn std::error::Error>> {
    ///     let n: i32 = arg.value().parse()?;
    ///     if n <= 0 {
    ///         return Err("number must be positive".into());
    ///     }
    ///     Ok(n)
    /// })?;
    /// # Ok::<(), noargs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - Returns [`Error::MissingArg`] if `self.is_present()` is `false` (argument is missing)
    /// - Returns [`Error::InvalidArg`] if `f(self)` returns `Err(_)` (validation or conversion failed)
    pub fn then<F, T, E>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        if !self.is_present() {
            return Err(Error::MissingArg {
                arg: Box::new(self),
            });
        }
        f(self.clone()).map_err(|e| Error::InvalidArg {
            arg: Box::new(self),
            reason: e.to_string(),
        })
    }

    /// Shorthand for `self.present().map(|arg| arg.then(f)).transpose()`.
    pub fn present_and_then<F, T, E>(self, f: F) -> Result<Option<T>, Error>
    where
        F: FnOnce(Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.present().map(|arg| arg.then(f)).transpose()
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
        let mut args = test_args(&["test", "foo", "bar"]);
        let arg = crate::arg("ARG");
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
        let mut args = test_args(&["test", "foo"]);
        let arg = crate::arg("ARG").default("bar");
        assert!(matches!(
            arg.take(&mut args),
            Arg::Positional { index: 1, .. }
        ));
        assert!(matches!(arg.take(&mut args), Arg::Default { .. }));
        assert!(matches!(arg.take(&mut args), Arg::Default { .. }));
    }

    #[test]
    fn example_arg() {
        let mut args = test_args(&["test", "foo"]);
        args.metadata_mut().help_mode = true;

        let arg = crate::arg("ARG").example("bar");
        assert!(matches!(arg.take(&mut args), Arg::Example { .. }));
        assert!(matches!(arg.take(&mut args), Arg::Example { .. }));
    }

    #[test]
    fn parse_arg() {
        let mut args = test_args(&["test", "1", "not a number"]);
        let arg = crate::arg("ARG");
        assert_eq!(
            arg.take(&mut args)
                .then(|a| a.value().parse::<usize>())
                .ok(),
            Some(1)
        );
        assert_eq!(
            arg.take(&mut args)
                .then(|a| a.value().parse::<usize>())
                .ok(),
            None
        );
        assert_eq!(
            arg.take(&mut args)
                .then(|a| a.value().parse::<usize>())
                .ok(),
            None
        );
    }

    fn test_args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }
}
