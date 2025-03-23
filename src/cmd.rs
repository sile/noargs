use crate::args::{Args, Metadata};

/// Specification for [`Cmd`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CmdSpec {
    /// Subcommand name (usually cebab-case).
    pub name: &'static str,

    /// Documentation.
    pub doc: &'static str,

    /// Minimum index that [`Cmd::index()`] can have.
    pub min_index: Option<usize>,

    /// Maximum index that [`Cmd::index()`] can have.
    pub max_index: Option<usize>,

    metadata: Metadata,
}

impl CmdSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self {
        name: "",
        doc: "",
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    /// Makes an [`CmdSpec`] instance with a specified name (equivalent to `noargs::cmd(name)`).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            ..Self::DEFAULT
        }
    }

    /// Updates the value of [`CmdSpec::doc`].
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }

    /// Updates the value of [`CmdSpec::min_index`].
    pub const fn min_index(mut self, index: Option<usize>) -> Self {
        self.min_index = index;
        self
    }

    /// Updates the value of [`CmdSpec::max_index`].
    pub const fn max_index(mut self, index: Option<usize>) -> Self {
        self.max_index = index;
        self
    }

    /// Takes the first [`Cmd`] instance that satisfies this specification from the raw arguments.
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

/// A subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Cmd {
    Some { spec: CmdSpec, index: usize },
    None { spec: CmdSpec },
}

impl Cmd {
    /// Returns the specification of this subcommand.
    pub fn spec(self) -> CmdSpec {
        match self {
            Cmd::Some { spec, .. } | Cmd::None { spec } => spec,
        }
    }

    /// Returns `Some(_)` if this subcommand is present.
    pub fn ok(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    /// Returns `true` if this subcommand is present.
    pub fn is_present(self) -> bool {
        matches!(self, Self::Some { .. })
    }

    /// Returns the index at which the raw value associated with this subcommand was located in [`Args`].
    pub fn index(self) -> Option<usize> {
        if let Self::Some { index, .. } = self {
            Some(index)
        } else {
            None
        }
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
