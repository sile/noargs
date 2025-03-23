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
    ArgSpec {
        name,
        ..ArgSpec::DEFAULT
    }
}

/// Makes an [`OptSpec`] instance with a specified name.
pub const fn opt(name: &'static str) -> OptSpec {
    OptSpec {
        name,
        ..OptSpec::DEFAULT
    }
}

/// Makes a [`FlagSpec`] instance with a specified name.
pub const fn flag(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        ..FlagSpec::DEFAULT
    }
}

/// Makes a [`CmdSpec`] instance with a specified name.
pub const fn cmd(name: &'static str) -> CmdSpec {
    CmdSpec {
        name,
        ..CmdSpec::DEFAULT
    }
}
