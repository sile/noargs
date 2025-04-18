use crate::{
    args::{Metadata, RawArgs},
    error::Error,
};

/// Specification for [`Opt`].
///
/// Note that `noargs` does not support options with only short names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptSpec {
    /// Option long name (usually kebab-case).
    pub name: &'static str,

    /// Option short name.
    pub short: Option<char>,

    /// Value type.
    pub ty: &'static str,

    /// Documentation.
    pub doc: &'static str,

    /// Environment variable name.
    ///
    /// If a non-empty value is set for this environment variable,
    /// it will be used as the value of this option when the option is not specified in [`RawArgs`].
    pub env: Option<&'static str>,

    /// Default value.
    pub default: Option<&'static str>,

    /// Example value (if this is set, the option is considered to be requried when generating the help text).
    ///
    /// This is only used if `RawArgs::metadata().help_mode` is `true`.
    pub example: Option<&'static str>,

    /// Minimum index that [`Opt::index()`] can have.
    pub min_index: Option<usize>,

    /// Maximum index that [`Opt::index()`] can have.
    pub max_index: Option<usize>,
}

impl OptSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "",
        short: None,
        ty: "VALUE",
        doc: "",
        env: None,
        default: None,
        example: None,
        min_index: None,
        max_index: None,
    };

    /// Makes an [`OptSpec`] instance with a specified name (equivalent to `noargs::opt(name)`).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            ..Self::DEFAULT
        }
    }

    /// Updates the value of [`OptSpec::short`].
    pub const fn short(mut self, name: char) -> Self {
        self.short = Some(name);
        self
    }

    /// Updates the value of [`OptSpec::ty`].
    pub const fn ty(mut self, value_type: &'static str) -> Self {
        self.ty = value_type;
        self
    }

    /// Updates the value of [`OptSpec::doc`].
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }

    /// Updates the value of [`OptSpec::env`].
    pub const fn env(mut self, variable_name: &'static str) -> Self {
        self.env = Some(variable_name);
        self
    }

    /// Updates the value of [`OptSpec::default`].
    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    /// Updates the value of [`OptSpec::example`].
    pub const fn example(mut self, example: &'static str) -> Self {
        self.example = Some(example);
        self
    }

    /// Updates the value of [`OptSpec::min_index`].
    pub const fn min_index(mut self, index: Option<usize>) -> Self {
        self.min_index = index;
        self
    }

    /// Updates the value of [`OptSpec::max_index`].
    pub const fn max_index(mut self, index: Option<usize>) -> Self {
        self.max_index = index;
        self
    }

    /// Takes the first [`Opt`] instance that satisfies this specification from the raw arguments.
    pub fn take(self, args: &mut RawArgs) -> Opt {
        let metadata = args.metadata();
        args.with_record_opt(|args| {
            if args.metadata().help_mode {
                return if self.default.is_some() {
                    Opt::Default {
                        spec: self,
                        metadata,
                    }
                } else if self.example.is_some() {
                    Opt::Example {
                        spec: self,
                        metadata,
                    }
                } else {
                    Opt::None { spec: self }
                };
            }

            let mut pending = None;
            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                if let Some(mut pending) = pending.take() {
                    match &mut pending {
                        Opt::Long { value, .. } | Opt::Short { value, .. } => {
                            if let Some(v) = raw_arg.value.take() {
                                *value = v;
                            } else {
                                return Opt::MissingValue {
                                    spec: self,
                                    long: matches!(pending, Opt::Long { .. }),
                                };
                            }
                        }
                        _ => unreachable!(),
                    }
                    return pending;
                }

                let Some(value) = &mut raw_arg.value else {
                    continue;
                };
                if !value.starts_with('-') {
                    continue;
                }

                if let Some(value) = value.strip_prefix("--") {
                    // Long name option.
                    let Some(value) = value.strip_prefix(self.name) else {
                        continue;
                    };
                    match value.chars().next() {
                        None => {
                            raw_arg.value = None;
                            pending = Some(Opt::Long {
                                spec: self,
                                metadata,
                                index,
                                value: "".to_owned(),
                            });
                        }
                        Some('=') => {
                            let opt_value = value[1..].to_owned();
                            raw_arg.value = None;
                            return Opt::Long {
                                spec: self,
                                metadata,
                                index,
                                value: opt_value,
                            };
                        }
                        Some(_) => {}
                    }
                    continue;
                }

                // Short name option.
                let Some(value) = self.short.and_then(|c| value[1..].strip_prefix(c)) else {
                    continue;
                };
                let value = value.to_owned();
                if value.is_empty() {
                    raw_arg.value = None;
                    pending = Some(Opt::Short {
                        spec: self,
                        metadata,
                        index,
                        value: "".to_owned(),
                    });
                } else {
                    raw_arg.value = None;
                    return Opt::Short {
                        spec: self,
                        metadata,
                        index,
                        value,
                    };
                }
            }

            if pending.is_some() {
                Opt::MissingValue {
                    spec: self,
                    long: matches!(pending, Some(Opt::Long { .. })),
                }
            } else if let Some(value) = self
                .env
                .and_then(|name| std::env::var(name).ok())
                .filter(|v| !v.is_empty())
            {
                Opt::Env {
                    spec: self,
                    metadata,
                    value,
                }
            } else if self.default.is_some() {
                Opt::Default {
                    spec: self,
                    metadata,
                }
            } else if self.example.is_some() && args.metadata().help_mode {
                Opt::Example {
                    spec: self,
                    metadata,
                }
            } else {
                Opt::None { spec: self }
            }
        })
    }
}

impl Default for OptSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A named argument with value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Opt {
    Long {
        spec: OptSpec,
        metadata: Metadata,
        index: usize,
        value: String,
    },
    Short {
        spec: OptSpec,
        metadata: Metadata,
        index: usize,
        value: String,
    },
    Env {
        spec: OptSpec,
        metadata: Metadata,
        value: String,
    },
    Default {
        spec: OptSpec,
        metadata: Metadata,
    },
    Example {
        spec: OptSpec,
        metadata: Metadata,
    },
    MissingValue {
        spec: OptSpec,
        long: bool,
    },
    None {
        spec: OptSpec,
    },
}

impl Opt {
    /// Parse the value of this option.
    #[deprecated(since = "0.3.0", note = "please use `then()` instead")]
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.clone().then(|opt| opt.value().parse())
    }

    /// Parse the value of this option if it is present.
    #[deprecated(since = "0.3.0", note = "please use `present_and_then()` instead")]
    pub fn parse_if_present<T>(&self) -> Result<Option<T>, Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.clone().present_and_then(|opt| opt.value().parse())
    }

    /// Similar to [`Opt::parse()`], but more flexible as this method allows you to specify an arbitrary parsing function.
    #[deprecated(since = "0.3.0", note = "please use `then()` instead")]
    pub fn parse_with<F, T, E>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.clone().then(|opt| f(&opt))
    }

    /// Returns the specification of this option.
    pub fn spec(&self) -> OptSpec {
        match self {
            Opt::Long { spec, .. }
            | Opt::Short { spec, .. }
            | Opt::Env { spec, .. }
            | Opt::Default { spec, .. }
            | Opt::Example { spec, .. }
            | Opt::MissingValue { spec, .. }
            | Opt::None { spec } => *spec,
        }
    }

    /// Returns `true` if this option is present.
    pub fn is_present(&self) -> bool {
        !matches!(self, Opt::None { .. })
    }

    /// Returns `true` if this option is present and has a value.
    pub fn is_value_present(&self) -> bool {
        !matches!(self, Opt::None { .. } | Opt::MissingValue { .. })
    }

    /// Returns `Some(self)` if this option is present.
    pub fn present(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    /// Applies additional conversion or validation to the option.
    ///
    /// This method allows for chaining transformations and validations when an option is present.
    /// It first checks if the option has a value and then applies the provided function.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut args = noargs::RawArgs::new(["example", "--num=42"].iter().map(|a| a.to_string()));
    /// let opt = noargs::opt("num").take(&mut args);
    ///
    /// // Parse as number and ensure it's positive
    /// let num = opt.then(|opt| -> Result<_, Box<dyn std::error::Error>> {
    ///     let n: i32 = opt.value().parse()?;
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
    /// - Returns [`Error::MissingOpt`] if `self.is_value_present()` is `false` (option is missing)
    /// - Returns [`Error::InvalidOpt`] if `f(self)` returns `Err(_)` (validation or conversion failed)
    pub fn then<F, T, E>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        if !self.is_value_present() {
            return Err(Error::MissingOpt {
                opt: Box::new(self),
            });
        }
        f(self.clone()).map_err(|e| Error::InvalidOpt {
            opt: Box::new(self),
            reason: e.to_string(),
        })
    }

    /// Shorthand for `self.present().map(|opt| opt.then(f)).transpose()`.
    pub fn present_and_then<F, T, E>(self, f: F) -> Result<Option<T>, Error>
    where
        F: FnOnce(Self) -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.present().map(|opt| opt.then(f)).transpose()
    }

    /// Returns the raw value of this option.
    #[deprecated(since = "0.3.0", note = "please use `present()` and `value()` instead")]
    pub fn raw_value(&self) -> Option<&str> {
        self.is_present().then_some(self.value())
    }

    /// Returns the raw value of this option, or an empty string if not present.
    #[deprecated(since = "0.3.0", note = "please use `value()` instead")]
    pub fn raw_value_or_empty(&self) -> &str {
        self.value()
    }

    /// Returns the raw value of this option, or an empty string if not present.
    pub fn value(&self) -> &str {
        match self {
            Opt::Long { value, .. } | Opt::Short { value, .. } | Opt::Env { value, .. } => value,
            Opt::Default { spec, .. } => spec.default.unwrap_or(""),
            Opt::Example { spec, .. } => spec.example.unwrap_or(""),
            Opt::MissingValue { .. } | Opt::None { .. } => "",
        }
    }

    /// Returns the index at which the raw value associated with the name of this option was located in [`RawArgs`].
    pub fn index(&self) -> Option<usize> {
        if let Opt::Long { index, .. } | Opt::Short { index, .. } = self {
            Some(*index)
        } else {
            None
        }
    }

    pub(crate) fn metadata(&self) -> Option<Metadata> {
        match self {
            Opt::Long { metadata, .. }
            | Opt::Short { metadata, .. }
            | Opt::Env { metadata, .. }
            | Opt::Default { metadata, .. }
            | Opt::Example { metadata, .. } => Some(*metadata),
            Opt::MissingValue { .. } | Opt::None { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_opt() {
        let mut args = args(&["test", "--foo", "bar", "-fbaz"]);
        let mut opt = opt("foo");
        opt.short = Some('f');
        assert!(matches!(opt.take(&mut args), Opt::Long { index: 1, .. }));
        assert!(matches!(opt.take(&mut args), Opt::Short { index: 3, .. }));
        assert!(matches!(opt.take(&mut args), Opt::None { .. }));
    }

    #[test]
    fn default_opt() {
        let mut args = args(&["test", "--foo=1", "--bar=2"]);
        let mut opt = opt("bar");
        opt.default = Some("3");
        assert!(matches!(opt.take(&mut args), Opt::Long { index: 2, .. }));
        assert!(matches!(opt.take(&mut args), Opt::Default { .. }));
        assert!(matches!(opt.take(&mut args), Opt::Default { .. }));
    }

    #[test]
    fn exampel_opt() {
        let mut args = args(&["test", "--foo=1", "--bar=2"]);
        args.metadata_mut().help_mode = true;

        let mut opt = opt("bar");
        opt.example = Some("3");
        assert!(matches!(opt.take(&mut args), Opt::Example { .. }));
        assert!(matches!(opt.take(&mut args), Opt::Example { .. }));
    }

    #[test]
    fn missing_short_opt_value() {
        let mut args = args(&["test", "-f"]);
        let mut opt = opt("foo");
        opt.short = Some('f');
        assert!(
            opt.take(&mut args)
                .present_and_then(|o| o.value().parse::<String>())
                .is_err()
        );
    }

    #[test]
    fn parse_opt() {
        let mut args = args(&["test", "--foo=1", "-f", "2", "--foo"]);
        let mut opt = opt("foo");
        opt.short = Some('f');
        assert_eq!(
            opt.take(&mut args)
                .then(|o| o.value().parse::<usize>())
                .ok(),
            Some(1)
        );
        assert_eq!(
            opt.take(&mut args)
                .then(|o| o.value().parse::<usize>())
                .ok(),
            Some(2)
        );
        assert_eq!(
            opt.take(&mut args)
                .then(|o| o.value().parse::<usize>())
                .ok(),
            None
        );
    }

    fn args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn opt(name: &'static str) -> OptSpec {
        OptSpec {
            name,
            ..Default::default()
        }
    }
}
