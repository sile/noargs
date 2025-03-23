use crate::args::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    pub name: &'static str,
    pub short: Option<char>,
    pub doc: &'static str,
    pub env: Option<&'static str>,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
}

impl FlagSpec {
    pub const DEFAULT: Self = Self {
        name: "",
        short: None,
        doc: "",
        env: None,
        min_index: None,
        max_index: None,
    };

    pub const HELP: Self = Self {
        name: "help",
        short: Some('h'),
        doc: "Print help",
        ..Self::DEFAULT
    };

    pub const VERSION: Self = Self {
        name: "version",
        doc: "Print version",
        ..Self::DEFAULT
    };

    pub const OPTIONS_END: Self = Self {
        name: "",
        doc: "Indicates that all arguments following this flag are positional",
        ..Self::DEFAULT
    };

    pub fn take(self, args: &mut Args) -> Flag {
        args.with_record_flag(|args| {
            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                let Some(value) = &mut raw_arg.value else {
                    continue;
                };
                if !value.starts_with('-') {
                    continue;
                }

                if value.starts_with("--") {
                    if &value[2..] == self.name {
                        raw_arg.value = None;
                        return Flag::Long { spec: self, index };
                    }
                } else if let Some(i) = value
                    .char_indices()
                    .skip(1)
                    .find_map(|(i, c)| (Some(c) == self.short).then_some(i))
                {
                    value.remove(i);
                    if value.len() == 1 {
                        raw_arg.value = None;
                    }
                    return Flag::Short { spec: self, index };
                }
            }

            if self
                .env
                .is_some_and(|name| std::env::var(name).is_ok_and(|v| !v.is_empty()))
            {
                Flag::Env { spec: self }
            } else {
                Flag::None { spec: self }
            }
        })
    }
}

impl Default for FlagSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    Long { spec: FlagSpec, index: usize },
    Short { spec: FlagSpec, index: usize },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_name_flag() {
        let mut args = args(&["test", "--foo"]);
        let flag = flag("foo");
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn short_name_flag() {
        let mut args = args(&["test", "-f", "-bf"]);

        let flag = short_flag('f');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 1, .. }));
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        let flag = short_flag('b');
        assert!(matches!(flag.take(&mut args), Flag::Short { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    #[test]
    fn env_flag() {
        let mut args = args(&["test", "--bar"]);

        let flag = FlagSpec {
            name: "foo",
            env: Some("TEST_ENV_FLAG_FOO"),
            ..Default::default()
        };
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        unsafe {
            std::env::set_var("TEST_ENV_FLAG_FOO", "1");
        }
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
        assert!(matches!(flag.take(&mut args), Flag::Env { .. }));
    }

    #[test]
    fn flag_range() {
        let mut args = args(&["test", "--foo", "--foo", "--foo"]);

        let mut flag = flag("foo");
        flag.min_index = Some(2);
        flag.max_index = Some(2);
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 2, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));

        flag.max_index = None;
        assert!(matches!(flag.take(&mut args), Flag::Long { index: 3, .. }));
        assert!(matches!(flag.take(&mut args), Flag::None { .. }));
    }

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn flag(long_name: &'static str) -> FlagSpec {
        FlagSpec {
            name: long_name,
            ..Default::default()
        }
    }

    fn short_flag(short_name: char) -> FlagSpec {
        FlagSpec {
            name: "dummy",
            short: Some(short_name),
            ..Default::default()
        }
    }
}
