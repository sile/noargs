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
            for (index, raw_arg) in args.raw_args_mut().iter_mut().enumerate() {
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
                let Some(short_char) = self.short else {
                    continue;
                };

                if let Some(value_after_dash) = value.strip_prefix('-') {
                    if let Some(value_after_short) = value_after_dash.strip_prefix(short_char) {
                        if value_after_short.is_empty() {
                            // Format: -f (value in next argument)
                            raw_arg.value = None;
                            pending = Some(Opt::Short {
                                spec: self,
                                metadata,
                                index,
                                value: "".to_owned(),
                            });
                        }
                        // Note: -fvalue format is intentionally not supported
                    }
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
        let mut args = test_args(&["test", "--foo", "bar", "-f", "baz"]);
        let mut opt = opt("foo");
        opt.short = Some('f');
        assert!(matches!(opt.take(&mut args), Opt::Long { index: 1, .. }));
        assert!(matches!(opt.take(&mut args), Opt::Short { index: 3, .. }));
        assert!(matches!(opt.take(&mut args), Opt::None { .. }));
    }

    #[test]
    fn default_opt() {
        let mut args = test_args(&["test", "--foo=1", "--bar=2"]);
        let mut opt = opt("bar");
        opt.default = Some("3");
        assert!(matches!(opt.take(&mut args), Opt::Long { index: 2, .. }));
        assert!(matches!(opt.take(&mut args), Opt::Default { .. }));
        assert!(matches!(opt.take(&mut args), Opt::Default { .. }));
    }

    #[test]
    fn example_opt() {
        let mut args = test_args(&["test", "--foo=1", "--bar=2"]);
        args.metadata_mut().help_mode = true;

        let mut opt = opt("bar");
        opt.example = Some("3");
        assert!(matches!(opt.take(&mut args), Opt::Example { .. }));
        assert!(matches!(opt.take(&mut args), Opt::Example { .. }));
    }

    #[test]
    fn missing_short_opt_value() {
        let mut args = test_args(&["test", "-f"]);
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
        let mut args = test_args(&["test", "--foo=1", "-f", "2", "--foo"]);
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

    #[test]
    fn short_option_equals_format_donot_works() {
        let mut args = test_args(&["test", "-f=value1", "-o=output.txt"]);

        let file_opt = opt("file").short('f');
        let result1 = file_opt.take(&mut args);
        assert!(matches!(result1, Opt::None { .. }));
        assert!(!result1.is_present());

        let output_opt = opt("output").short('o');
        let result2 = output_opt.take(&mut args);
        assert!(matches!(result2, Opt::None { .. }));
        assert!(!result2.is_present());
    }

    #[test]
    fn short_option_separate_value() {
        // Test that -f value format works
        let mut args = test_args(&["test", "-f", "value1"]);
        let file_opt = opt("file").short('f');
        let result = file_opt.take(&mut args);
        assert!(matches!(result, Opt::Short { .. }));
        assert_eq!(result.value(), "value1");
    }

    #[test]
    fn concatenated_short_option_no_longer_works() {
        // Test that -fvalue format (without =) no longer works
        let mut args = test_args(&["test", "-fvalue"]);
        let file_opt = opt("file").short('f');
        let result = file_opt.take(&mut args);
        assert!(matches!(result, Opt::None { .. }));
        assert!(!result.is_present());
    }

    #[test]
    fn all_supported_formats() {
        let mut args = test_args(&[
            "test",
            "--long=long_value", // --key=value
            "--other",
            "other_value", // --key value
            "-f",
            "file_value", // -k value (only supported short format)
        ]);

        let long_opt = opt("long");
        let result1 = long_opt.take(&mut args);
        assert!(matches!(result1, Opt::Long { .. }));
        assert_eq!(result1.value(), "long_value");

        let other_opt = opt("other");
        let result2 = other_opt.take(&mut args);
        assert!(matches!(result2, Opt::Long { .. }));
        assert_eq!(result2.value(), "other_value");

        let file_opt = opt("file").short('f');
        let result3 = file_opt.take(&mut args);
        assert!(matches!(result3, Opt::Short { .. }));
        assert_eq!(result3.value(), "file_value");
    }

    #[test]
    fn short_option_edge_cases() {
        // Test that -f= format should not match
        let mut args = test_args(&["test", "-f="]);
        let file_opt = opt("file").short('f');
        let result = file_opt.take(&mut args);
        assert!(matches!(result, Opt::None { .. }));
        assert!(!result.is_present());

        // Test that -f=--not-an-option should not match
        let mut args = test_args(&["test", "-f=--not-an-option"]);
        let file_opt = opt("file").short('f');
        let result = file_opt.take(&mut args);
        assert!(matches!(result, Opt::None { .. }));
        assert!(!result.is_present());
    }

    #[test]
    fn long_option_formats_still_work() {
        // Verify that long options still support both formats
        let mut args = test_args(&["test", "--file=value1", "--output", "value2"]);

        let file_opt = opt("file");
        let result1 = file_opt.take(&mut args);
        assert!(matches!(result1, Opt::Long { .. }));
        assert_eq!(result1.value(), "value1");

        let output_opt = opt("output");
        let result2 = output_opt.take(&mut args);
        assert!(matches!(result2, Opt::Long { .. }));
        assert_eq!(result2.value(), "value2");
    }

    fn test_args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn opt(name: &'static str) -> OptSpec {
        OptSpec {
            name,
            ..Default::default()
        }
    }
}
