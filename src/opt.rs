use std::str::FromStr;

use crate::{
    args::{Args, Metadata},
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptSpec {
    pub name: &'static str, // TODO: Option?
    pub short: Option<char>,
    pub ty: &'static str,
    pub doc: &'static str,
    pub env: Option<&'static str>,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
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
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Opt {
        self.metadata = args.metadata();
        args.log_mut().record_opt(self);

        let mut pending = None;
        for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
            if let Some(mut pending) = pending.take() {
                match &mut pending {
                    Opt::Long { value, .. } | Opt::Short { value, .. } => {
                        *value = raw_arg.value.take()
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

            if value.starts_with("--") {
                // Long name option.
                if !value[2..].starts_with(self.name) {
                    continue;
                }
                match value[2 + self.name.len()..].chars().next() {
                    None => {
                        pending = Some(Opt::Long {
                            spec: self,
                            index,
                            value: None,
                        });
                    }
                    Some('=') => {
                        let opt_value = value[2 + self.name.len() + 1..].to_owned();
                        raw_arg.value = None;
                        return Opt::Long {
                            spec: self,
                            index,
                            value: Some(opt_value),
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
                    pending = Some(Opt::Short {
                        spec: self,
                        index,
                        value: None,
                    });
                }
                Some('=') => {
                    let opt_name_len = self.short.map(|c| c.len_utf8()).unwrap_or(0);
                    let opt_value = value[1 + opt_name_len + 1..].to_owned();
                    raw_arg.value = None;
                    return Opt::Short {
                        spec: self,
                        index,
                        value: Some(opt_value),
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
            Opt::Env { spec: self, value }
        } else if self.default.is_some() {
            Opt::Default { spec: self }
        } else if self.example.is_some() {
            Opt::Example { spec: self }
        } else {
            Opt::None { spec: self }
        }
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
        index: usize,
        value: Option<String>,
    },
    Short {
        spec: OptSpec,
        index: usize,
        value: Option<String>,
    },
    Env {
        spec: OptSpec,
        value: String,
    },
    Default {
        spec: OptSpec,
    },
    Example {
        spec: OptSpec,
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
        let value = self
            .value()
            .ok_or_else(|| Error::MissingOpt { opt: self.clone() })?;
        value.parse::<T>().map_err(|e| Error::ParseOptError {
            opt: self.clone(),
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

    pub fn spec(&self) -> OptSpec {
        match self {
            Opt::Long { spec, .. }
            | Opt::Short { spec, .. }
            | Opt::Env { spec, .. }
            | Opt::Default { spec }
            | Opt::Example { spec }
            | Opt::None { spec } => *spec,
        }
    }

    pub fn is_present(&self) -> bool {
        !matches!(self, Opt::None { .. })
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Opt::Long { value, .. } | Opt::Short { value, .. } => {
                value.as_ref().map(|v| v.as_str())
            }
            Opt::Env { value, .. } => Some(value),
            Opt::Default { spec } => spec.default,
            Opt::Example { spec } => spec.example,
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
