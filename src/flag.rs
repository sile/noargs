use crate::args::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    pub long: &'static str,
    pub short: char,
    pub doc: &'static str,
    pub env: &'static str,
    pub before: Option<usize>,
    pub after: Option<usize>,
}

impl FlagSpec {
    pub const DEFAULT: Self = Self {
        long: "",
        short: '\0',
        doc: "",
        env: "",
        before: None,
        after: None,
    };

    pub const HELP: Self = Self {
        long: "help",
        short: 'h',
        doc: "Print help",
        ..Self::DEFAULT
    };

    pub const VERSION: Self = Self {
        long: "version",
        doc: "Print version",
        ..Self::DEFAULT
    };

    pub const OPTIONS_END: Self = Self {
        long: "",
        doc: "Indicates that all arguments following this flag are positional",
        ..Self::DEFAULT
    };

    pub fn take(self, args: &mut Args) -> Flag {
        for (index, raw_arg) in args.raw_args_mut().iter_mut().enumerate() {
            let Some(value) = &mut raw_arg.value else {
                continue;
            };
            if !value.starts_with('-') {
                continue;
            }

            if value.starts_with("--") {
                if &value[2..] == self.long {
                    raw_arg.value = None;
                    return Flag::Long { spec: self, index };
                }
            } else if let Some(i) = value
                .char_indices()
                .skip(1)
                .find_map(|(i, c)| (c == self.short).then_some(i))
            {
                value.remove(i);
                if value.len() == 1 {
                    raw_arg.value = None;
                }
                return Flag::Short { spec: self, index };
            }
        }

        if std::env::var(self.env).is_ok_and(|v| !v.is_empty()) {
            Flag::Env { spec: self }
        } else {
            Flag::None { spec: self }
        }
    }
}

impl Default for FlagSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    Short { spec: FlagSpec, index: usize },
    Long { spec: FlagSpec, index: usize },
    Env { spec: FlagSpec },
    None { spec: FlagSpec },
}

impl Flag {
    pub fn spec(self) -> FlagSpec {
        match self {
            Flag::Short { spec, .. }
            | Flag::Long { spec, .. }
            | Flag::Env { spec }
            | Flag::None { spec } => spec,
        }
    }

    pub fn is_present(self) -> bool {
        !matches!(self, Flag::None { .. })
    }

    pub fn index(self) -> Option<usize> {
        match self {
            Flag::Short { index, .. } | Flag::Long { index, .. } => Some(index),
            Flag::Env { .. } | Flag::None { .. } => None,
        }
    }

    pub fn ok(self) -> Option<Flag> {
        self.is_present().then_some(self)
    }
}
