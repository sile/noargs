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
        args.log_mut().record_arg(self);

        for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
            if let Some(value) = raw_arg.value.take() {
                return Arg::Positional {
                    spec: self,
                    index,
                    value,
                };
            };
        }

        if self.default.is_some() {
            Arg::Default { spec: self }
        } else if self.example.is_some() {
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
        value: String,
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
    pub fn parse<T: FromStr>(&self) -> Result<T, Error> {
        todo!()
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

    pub fn value(&self) -> Option<&str> {
        match self {
            Arg::Positional { value, .. } => Some(value.as_str()),
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
