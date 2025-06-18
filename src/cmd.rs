use crate::args::RawArgs;

/// Specification for [`Cmd`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CmdSpec {
    /// Subcommand name (usually cebab-case).
    pub name: &'static str,

    /// Documentation.
    pub doc: &'static str,
}

impl CmdSpec {
    /// The default specification.
    pub const DEFAULT: Self = Self { name: "", doc: "" };

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

    /// Takes the first [`Cmd`] instance that satisfies this specification from the raw arguments.
    pub fn take(self, args: &mut RawArgs) -> Cmd {
        args.with_record_cmd(|args| {
            for (index, raw_arg) in args.raw_args_mut().iter_mut().enumerate() {
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

    /// Returns `true` if this subcommand is present.
    pub fn is_present(self) -> bool {
        matches!(self, Self::Some { .. })
    }

    /// Returns `Some(self)` if this subcommand is present.
    pub fn present(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    /// Returns the index at which the raw value associated with this subcommand was located in [`RawArgs`].
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
        if let Some(_cmd) = cmd("bar").take(&mut args).present() {
            panic!();
        } else if let Some(_cmd) = cmd("run").take(&mut args).present() {
            let flag = FlagSpec {
                name: "foo",
                ..Default::default()
            };
            assert!(matches!(flag.take(&mut args), Flag::Long { index: 1, .. }));
            assert!(matches!(flag.take(&mut args), Flag::Long { index: 3, .. }));
            assert!(matches!(flag.take(&mut args), Flag::None { .. }));
        } else {
            panic!()
        }
    }

    fn args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn cmd(name: &'static str) -> CmdSpec {
        CmdSpec {
            name,
            ..Default::default()
        }
    }
}
