use std::str::FromStr;

use crate::{
    args::{Args, Metadata},
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptSpec {
    pub name: &'static str,
    pub short: Option<char>,
    pub ty: &'static str,
    pub doc: &'static str,
    pub env: Option<&'static str>,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
}

impl OptSpec {
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

    pub fn take(self, args: &mut Args) -> Opt {
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
                        Opt::Long {
                            raw_value: value, ..
                        }
                        | Opt::Short {
                            raw_value: value, ..
                        } => *value = raw_arg.value.take(),
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
                                raw_value: None,
                            });
                        }
                        Some('=') => {
                            let opt_value = value[1..].to_owned();
                            raw_arg.value = None;
                            return Opt::Long {
                                spec: self,
                                metadata,
                                index,
                                raw_value: Some(opt_value),
                            };
                        }
                        Some(_) => {}
                    }
                    continue;
                } else if value[1..].chars().next() != self.short {
                    continue;
                }

                // Short name option.
                match value[1..].chars().nth(1) {
                    None => {
                        raw_arg.value = None;
                        pending = Some(Opt::Short {
                            spec: self,
                            metadata,
                            index,
                            raw_value: None,
                        });
                    }
                    Some('=') => {
                        let opt_name_len = self.short.map(|c| c.len_utf8()).unwrap_or(0);
                        let opt_value = value[1 + opt_name_len + 1..].to_owned();
                        raw_arg.value = None;
                        return Opt::Short {
                            spec: self,
                            metadata,
                            index,
                            raw_value: Some(opt_value),
                        };
                    }
                    Some(_) => {}
                }
            }

            if let Some(value) = self
                .env
                .and_then(|name| std::env::var(name).ok())
                .filter(|v| !v.is_empty())
            {
                Opt::Env {
                    spec: self,
                    metadata,
                    raw_value: value,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opt {
    Long {
        spec: OptSpec,
        metadata: Metadata,
        index: usize,
        raw_value: Option<String>,
    },
    Short {
        spec: OptSpec,
        metadata: Metadata,
        index: usize,
        raw_value: Option<String>,
    },
    Env {
        spec: OptSpec,
        metadata: Metadata,
        raw_value: String,
    },
    Default {
        spec: OptSpec,
        metadata: Metadata,
    },
    Example {
        spec: OptSpec,
        metadata: Metadata,
    },
    None {
        spec: OptSpec,
    },
}

impl Opt {
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let value = self.raw_value().ok_or_else(|| Error::MissingOpt {
            opt: Box::new(self.clone()),
        })?;
        value.parse::<T>().map_err(|e| Error::ParseOptError {
            opt: Box::new(self.clone()),
            reason: e.to_string(),
        })
    }

    pub fn parse_if_present<T>(&self) -> Result<Option<T>, Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.is_present().then(|| self.parse()).transpose()
    }

    pub fn parse_with<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Self) -> Result<T, Error>,
    {
        f(self)
    }

    pub fn spec(&self) -> OptSpec {
        match self {
            Opt::Long { spec, .. }
            | Opt::Short { spec, .. }
            | Opt::Env { spec, .. }
            | Opt::Default { spec, .. }
            | Opt::Example { spec, .. }
            | Opt::None { spec } => *spec,
        }
    }

    pub fn is_present(&self) -> bool {
        !matches!(self, Opt::None { .. })
    }

    pub fn raw_value(&self) -> Option<&str> {
        match self {
            Opt::Long { raw_value, .. } | Opt::Short { raw_value, .. } => {
                raw_value.as_ref().map(|v| v.as_str())
            }
            Opt::Env { raw_value, .. } => Some(raw_value),
            Opt::Default { spec, .. } => spec.default,
            Opt::Example { spec, .. } => spec.example,
            Opt::None { .. } => None,
        }
    }

    pub fn index(&self) -> Option<usize> {
        if let Opt::Long { index, .. } | Opt::Short { index, .. } = self {
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
    fn required_opt() {
        let mut args = args(&["test", "--foo", "bar", "-f=baz"]);
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
    fn parse_opt() {
        let mut args = args(&["test", "--foo=1", "-f", "2", "--foo"]);
        let mut opt = opt("foo");
        opt.short = Some('f');
        assert_eq!(opt.take(&mut args).parse::<usize>().ok(), Some(1));
        assert_eq!(opt.take(&mut args).parse::<usize>().ok(), Some(2));
        assert_eq!(opt.take(&mut args).parse::<usize>().ok(), None);
    }

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn opt(name: &'static str) -> OptSpec {
        OptSpec {
            name,
            ..Default::default()
        }
    }
}
