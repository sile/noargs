use crate::args::{Args, Metadata};

// TODO: s/SubcommandSpec/CmdSpec/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubcommandSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
}

impl SubcommandSpec {
    pub const DEFAULT: Self = Self {
        name: "",
        doc: "",
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Subcommand {
        self.metadata = args.metadata();
        args.with_record_subcommand(|args| {
            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                let Some(value) = &raw_arg.value else {
                    continue;
                };

                if value == self.name {
                    raw_arg.value = None;
                    return Subcommand::Some { spec: self, index };
                }
            }

            Subcommand::None { spec: self }
        })
    }
}

impl Default for SubcommandSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subcommand {
    Some { spec: SubcommandSpec, index: usize },
    None { spec: SubcommandSpec },
}

impl Subcommand {
    pub fn spec(self) -> SubcommandSpec {
        match self {
            Subcommand::Some { spec, .. } | Subcommand::None { spec } => spec,
        }
    }

    pub fn ok(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    pub fn index(self) -> Option<usize> {
        if let Self::Some { index, .. } = self {
            Some(index)
        } else {
            None
        }
    }

    pub fn is_present(self) -> bool {
        matches!(self, Self::Some { .. })
    }
}

#[cfg(test)]
mod tests {
    use crate::flag::{Flag, FlagSpec};

    use super::*;

    #[test]
    fn subcommand_and_flag() {
        let mut args = args(&["test", "--foo", "run", "--foo"]);
        if let Some(_cmd) = subcommand("bar").take(&mut args).ok() {
            panic!();
        } else if let Some(cmd) = subcommand("run").take(&mut args).ok() {
            let flag = FlagSpec {
                name: "foo",
                min_index: cmd.index(),
                ..Default::default()
            };
            assert!(matches!(flag.take(&mut args), Flag::Long { index: 3, .. }));
            assert!(matches!(flag.take(&mut args), Flag::None { .. }));
        } else {
            panic!()
        }
    }

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn subcommand(name: &'static str) -> SubcommandSpec {
        SubcommandSpec {
            name,
            ..Default::default()
        }
    }
}
