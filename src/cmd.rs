use crate::args::{Args, Metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CmdSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
}

impl CmdSpec {
    pub const DEFAULT: Self = Self {
        name: "",
        doc: "",
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Cmd {
        self.metadata = args.metadata();
        args.with_record_cmd(|args| {
            for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
                let Some(value) = &raw_arg.value else {
                    continue;
                };

                if value == self.name {
                    raw_arg.value = None;
                    return Cmd::Some { spec: self, index };
                }
            }

            Cmd::None { spec: self }
        })
    }
}

impl Default for CmdSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmd {
    Some { spec: CmdSpec, index: usize },
    None { spec: CmdSpec },
}

impl Cmd {
    pub fn spec(self) -> CmdSpec {
        match self {
            Cmd::Some { spec, .. } | Cmd::None { spec } => spec,
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
    fn cmd_and_flag() {
        let mut args = args(&["test", "--foo", "run", "--foo"]);
        if let Some(_cmd) = cmd("bar").take(&mut args).ok() {
            panic!();
        } else if let Some(cmd) = cmd("run").take(&mut args).ok() {
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

    fn cmd(name: &'static str) -> CmdSpec {
        CmdSpec {
            name,
            ..Default::default()
        }
    }
}
