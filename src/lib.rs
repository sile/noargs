mod arg;
mod args;
mod cmd;
mod error;
mod flag;
mod formatter;
mod help;
mod opt;

pub use self::arg::{Arg, ArgSpec};
pub use self::args::{Args, Metadata};
pub use self::cmd::{Cmd, CmdSpec};
pub use self::error::{Error, Result};
pub use self::flag::{Flag, FlagSpec};
pub use self::opt::{Opt, OptSpec};

/// Returns an [`Args`] instance initialized with command-line arguments.
///
/// This is a shorthand for `Args::new(std::env::args())`.
pub fn args() -> Args {
    Args::new(std::env::args())
}

// TODO: arg, opt, flag, cmd
