//! Imperative command-line argument parser library with no dependencies, no macros, and no implicit I/O.
//!
//! # Features
//!
//! - Supports the following argument types:
//!   - Positional arguments ([`Arg`])
//!   - Named arguments with values ([`Opt`])
//!   - Named arguments without values ([`Flag`])
//!   - Subcommands ([`Cmd`])
//! - Automatically generates help text
//! - Simple and minimal interface due to its imperative nature (no complex DSL)
#![warn(missing_docs)]
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
pub use self::error::Error;
pub use self::flag::{Flag, FlagSpec};
pub use self::opt::{Opt, OptSpec};

/// A specialized [`std::result::Result`] type for the [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

/// Makes an [`Args`] instance initialized with command-line arguments.
///
/// This is a shorthand for `Args::new(std::env::args())`.
pub fn args() -> Args {
    Args::new(std::env::args())
}

/// Makes an [`ArgSpec`] instance with a specified name.
pub const fn arg(name: &'static str) -> ArgSpec {
    ArgSpec::new(name)
}

/// Makes an [`OptSpec`] instance with a specified name.
pub const fn opt(name: &'static str) -> OptSpec {
    OptSpec::new(name)
}

/// Makes a [`FlagSpec`] instance with a specified name.
pub const fn flag(name: &'static str) -> FlagSpec {
    FlagSpec::new(name)
}

/// Makes a [`CmdSpec`] instance with a specified name.
pub const fn cmd(name: &'static str) -> CmdSpec {
    CmdSpec::new(name)
}

/// Well-known flag (`--help, -h`) for printing help information.
pub const HELP_FLAG: FlagSpec = flag("help").short('h').doc("Print help");

/// Well-known flag (`--version`) for printing version information.
pub const VERSION_FLAG: FlagSpec = flag("version").doc("Print version");

/// Well-known flag (`--`) to indicate the end of options (named arguments).
pub const OPTIONS_END_FLAG: FlagSpec =
    flag("").doc("Indicates that all arguments following this flag are positional");
