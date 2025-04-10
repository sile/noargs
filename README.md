noargs
======

[![noargs](https://img.shields.io/crates/v/noargs.svg)](https://crates.io/crates/noargs)
[![Documentation](https://docs.rs/noargs/badge.svg)](https://docs.rs/noargs)
[![Actions Status](https://github.com/sile/noargs/workflows/CI/badge.svg)](https://github.com/sile/noargs/actions)
![License](https://img.shields.io/crates/l/noargs)

`noargs` is an imperative command-line argument parser library for Rust with no dependencies, no macros, and no implicit I/O.

Features
--------

- Supports the following argument types:
  - Positional arguments ([`Arg`])
  - Named arguments with values ([`Opt`])
  - Named arguments without values ([`Flag`])
  - Subcommands ([`Cmd`])
- Automatically generates help text
- Simple and minimal interface due to its imperative nature (no complex DSL)

[`Arg`]: https://docs.rs/noargs/latest/noargs/struct.Arg.html
[`Opt`]: https://docs.rs/noargs/latest/noargs/struct.Opt.html
[`Flag`]: https://docs.rs/noargs/latest/noargs/struct.Flag.html
[`Cmd`]: https://docs.rs/noargs/latest/noargs/struct.Cmd.html

Examples
--------

The following code demonstrates the basic usage of `noargs`:
```rust
fn main() -> noargs::Result<()> {
    // Create `noargs::RawArgs` having the result of `std::env::args()`.
    let mut args = noargs::raw_args();

    // Set metadata for help.
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    // Handle well-known flags.
    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // Handle application specific args.
    let foo: usize = noargs::opt("foo").default("1").take(&mut args).parse()?;
    let bar: bool = noargs::flag("bar").take(&mut args).is_present();
    let baz: Option<String> = noargs::arg("[BAZ]").take(&mut args).parse_if_present()?;

    // Check unexpected args and build help text if need.
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    // Do application logic.

    Ok(())
```
