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

### Basic Usage

The following code demonstrates the basic usage of `noargs`:

```rust
fn main() -> noargs::Result<()> {
    // Create `noargs::RawArgs` having the result of `std::env::args()`.
    let mut args = noargs::raw_args();

    // Set metadata for help
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    // Handle well-known flags
    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // Handle application specific args
    let foo: usize = noargs::opt("foo")
        .default("1").take(&mut args).then(|o| o.value().parse())?;
    let bar: bool = noargs::flag("bar")
        .take(&mut args).is_present();
    let baz: Option<String> = noargs::arg("[BAZ]")
        .take(&mut args).present_and_then(|a| a.value().parse())?;

    // Check unexpected args and build help text if need
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    // Do application logic
    println!("foo: {}, bar: {}, baz: {:?}", foo, bar, baz);

    Ok(())
}
```

For a fuller no-subcommand example (with common pitfalls), see
[`examples/basics.rs`](examples/basics.rs).
For repeated options / positional arrays, see
[`examples/arrays.rs`](examples/arrays.rs).

### Subcommands

The following example shows how to handle subcommands.
For a fuller command-routing pattern, see
[`examples/subcommands.rs`](examples/subcommands.rs).

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    // Handle well-known flags
    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // Handle subcommands.
    //
    // `args` is moved into the matching handler, so each arm leaves with `return`.
    if noargs::cmd("start")
        .doc("Start the service")
        .take(&mut args)
        .is_present()
    {
        return run_start(args);
    }
    if noargs::cmd("stop")
        .doc("Stop the service")
        .take(&mut args)
        .is_present()
    {
        // No application specific args: `finish` still runs here, not in `main`.
        if let Some(help) = args.finish()? {
            print!("{help}");
            return Ok(());
        }
        println!("Stopping service");
        return Ok(());
    }

    // No subcommand: top-level help, and the top-level argument checks.
    if let Some(help) = args.finish()? {
        print!("{help}");
    }

    Ok(())
}

fn run_start(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let port: u16 = noargs::opt("port")
        .short('p')
        .default("8080")
        .take(&mut args)
        .then(|o| o.value().parse())?;

    // Help text and the `start` argument checks, before any work is done.
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    println!("Starting service on port {}", port);
    Ok(())
}
```

A subcommand handler takes `RawArgs` by value and calls `finish` itself, after
its `take`s and before it does any work. A stray argument is then reported
before the command touches anything, and `Some(help)` is the only branch the
handler needs.
