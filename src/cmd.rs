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

                // Ensure only the next unconsumed argument is processed as a subcommand.
                break;
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
    use crate::flag::Flag;

    use super::*;

    #[test]
    fn cmd_at_first_position() {
        let mut args = test_args(&["test", "run", "--foo", "test", "--foo"]);
        let cmd = crate::cmd("run").take(&mut args);

        assert!(cmd.is_present());
        assert_eq!(cmd.index(), Some(1));
        assert_eq!(cmd.spec().name, "run");
    }

    #[test]
    fn cmd_not_at_first_position_not_found() {
        let mut args = test_args(&["test", "--foo", "run", "--foo"]);

        // First, try to find 'run' command - it won't be found due to '--foo' being in the way
        let cmd = crate::cmd("run").take(&mut args);
        assert!(!cmd.is_present());
        assert_eq!(cmd.index(), None);

        // But if we consume the first '--foo' flag first...
        let mut args = test_args(&["test", "--foo", "run", "--foo"]);
        let flag = crate::flag("foo");
        let first_flag = flag.take(&mut args);
        assert!(matches!(first_flag, Flag::Long { index: 1, .. }));

        // Now the 'run' command can be found
        let cmd = crate::cmd("run").take(&mut args);
        assert!(cmd.is_present());
        assert_eq!(cmd.index(), Some(2));
    }

    #[test]
    fn cmd_not_found() {
        let mut args = test_args(&["test", "--foo", "run", "--foo"]);
        let cmd = crate::cmd("nonexistent").take(&mut args);

        assert!(!cmd.is_present());
        assert_eq!(cmd.index(), None);
    }

    #[test]
    fn cmd_consumed_after_take() {
        let mut args = test_args(&["test", "run", "--foo"]);
        let cmd = crate::cmd("run").take(&mut args);

        assert!(cmd.is_present());

        // Taking the same command again should not find it (already consumed)
        let cmd2 = crate::cmd("run").take(&mut args);
        assert!(!cmd2.is_present());
    }

    #[test]
    fn cmd_with_flags() {
        let mut args = test_args(&["test", "run", "--foo", "--bar"]);
        let cmd = crate::cmd("run").take(&mut args);

        assert!(cmd.is_present());

        // Flags should still be available after command is consumed
        let flag1 = crate::flag("foo");
        assert!(matches!(flag1.take(&mut args), Flag::Long { index: 2, .. }));

        let flag2 = crate::flag("bar");
        assert!(matches!(flag2.take(&mut args), Flag::Long { index: 3, .. }));
    }

    #[test]
    fn multiple_commands() {
        let mut args = test_args(&["test", "first", "second", "third"]);

        let cmd1 = crate::cmd("first").take(&mut args);
        assert!(cmd1.is_present());

        let cmd2 = crate::cmd("second").take(&mut args);
        assert!(cmd2.is_present());

        let cmd3 = crate::cmd("third").take(&mut args);
        assert!(cmd3.is_present());
    }

    #[test]
    fn cmd_methods() {
        let mut args = test_args(&["test", "run"]);
        let cmd = crate::cmd("run").take(&mut args);

        // Test present command methods
        assert!(cmd.is_present());
        assert!(cmd.present().is_some());
        assert_eq!(cmd.index(), Some(1));
        assert_eq!(cmd.spec().name, "run");

        // Test absent command methods
        let mut args2 = test_args(&["other"]);
        let cmd2 = crate::cmd("run").take(&mut args2);

        assert!(!cmd2.is_present());
        assert!(cmd2.present().is_none());
        assert_eq!(cmd2.index(), None);
        assert_eq!(cmd2.spec().name, "run");
    }

    #[test]
    fn cmd_with_empty_args() {
        let mut args = test_args(&["test"]);
        let cmd = crate::cmd("run").take(&mut args);

        assert!(!cmd.is_present());
        assert_eq!(cmd.index(), None);
    }

    fn test_args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }
}
