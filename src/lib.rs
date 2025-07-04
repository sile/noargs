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
//!
//! # Examples
//!
//! The following code demonstrates the basic usage of `noargs`:
//! ```
//! fn main() -> noargs::Result<()> {
//!     // Create `noargs::RawArgs` having the result of `std::env::args()`.
//!     let mut args = noargs::raw_args();
//!
//!     // Set metadata for help
//!     args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
//!     args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");
//!
//!     // Handle well-known flags
//!     if noargs::VERSION_FLAG.take(&mut args).is_present() {
//!         println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
//!         return Ok(());
//!     }
//!     noargs::HELP_FLAG.take_help(&mut args);
//!
//!     // Handle application specific args
//!     let foo: usize = noargs::opt("foo")
//!         .default("1").take(&mut args).then(|a| a.value().parse())?;
//!     let bar: bool = noargs::flag("bar")
//!         .take(&mut args).is_present();
//!     let baz: Option<String> = noargs::arg("[BAZ]")
//!         .take(&mut args).present_and_then(|a| a.value().parse())?;
//!
//!     // Check unexpected args and build help text if need
//!     if let Some(help) = args.finish()? {
//!         print!("{help}");
//!         return Ok(());
//!     }
//!
//!     // Do application logic
//!
//!     Ok(())
//! }
//! ```
//!
//! The following example shows how to handle subcommands:
//! ```
//! fn main() -> noargs::Result<()> {
//!     let mut args = noargs::raw_args();
//!     args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
//!     args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");
//!
//!     // Handle well-known flags
//!     if noargs::VERSION_FLAG.take(&mut args).is_present() {
//!         println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
//!         return Ok(());
//!     }
//!     noargs::HELP_FLAG.take_help(&mut args);
//!     # args.metadata_mut().help_mode = true;
//!
//!     // Handle subcommands
//!     if noargs::cmd("start")
//!         .doc("Start the service")
//!         .take(&mut args)
//!         .is_present()
//!     {
//!         let port: u16 = noargs::opt("port")
//!             .short('p')
//!             .default("8080")
//!             .take(&mut args)
//!             .then(|o| o.value().parse())?;
//!
//!         println!("Starting service on port {}", port);
//!     } else if noargs::cmd("stop")
//!         .doc("Stop the service")
//!         .take(&mut args)
//!         .is_present()
//!     {
//!         println!("Stopping service");
//!     } else if let Some(help) = args.finish()? {
//!         print!("{help}");
//!         return Ok(());
//!     }
//!
//!     Ok(())
//! }
//! ```
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
pub use self::args::{Metadata, RawArgs};
pub use self::cmd::{Cmd, CmdSpec};
pub use self::error::Error;
pub use self::flag::{Flag, FlagSpec};
pub use self::opt::{Opt, OptSpec};

/// A specialized [`std::result::Result`] type for the [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

/// Makes an [`RawArgs`] instance initialized with command-line arguments.
///
/// This is a shorthand for `RawArgs::new(std::env::args())`.
pub fn raw_args() -> RawArgs {
    RawArgs::new(std::env::args())
}

/// Makes an [`ArgSpec`] instance with a specified name.
///
/// # Recommended Naming Convention
///
/// - Required: `<NAME>`
/// - Optional: `[NAME]`
/// - Zero or more: `[NAME]...`
/// - One or more: `<NAME>...`
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
pub const HELP_FLAG: FlagSpec = flag("help")
    .short('h')
    .doc("Print help ('--help' for full help, '-h' for summary)");

/// Well-known flag (`--version`) for printing version information.
pub const VERSION_FLAG: FlagSpec = flag("version").doc("Print version");
