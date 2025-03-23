use std::str::FromStr;

use crate::{
    args::{Args, Metadata},
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgSpec {
    pub name: &'static str,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
    pub doc: &'static str,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
}

impl ArgSpec {
    pub const DEFAULT: Self = Self {
        name: "ARGUMENT",
        default: None,
        example: None,
        doc: "",
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Arg {
        self.metadata = args.metadata();
        args.record_arg(self);

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
        } else if self.example.is_some() && args.metadata().help_mode {
            Arg::Example { spec: self }
        } else {
            Arg::None { spec: self }
        }
    }
}

impl Default for ArgSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let value = self
            .raw_value()
            .ok_or_else(|| Error::MissingArg { arg: self.spec() })?;
        value.parse::<T>().map_err(|e| Error::ParseArgError {
            arg: self.spec(),
            value: value.to_owned(),
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

    pub fn spec(&self) -> ArgSpec {
        match self {
            Arg::Positional { spec, .. }
            | Arg::Default { spec }
            | Arg::Example { spec }
            | Arg::None { spec } => *spec,
        }
    }

    pub fn is_present(&self) -> bool {
        !matches!(self, Self::None { .. })
    }

    pub fn raw_value(&self) -> Option<&str> {
        match self {
            Arg::Positional { raw_value, .. } => Some(raw_value.as_str()),
            Arg::Default { spec } => spec.default,
            Arg::Example { spec } => spec.example,
            Arg::None { .. } => None,
        }
    }

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
        assert!(matches!(
            arg.take(&mut args),
            Arg::Positional { index: 1, .. }
        ));
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
