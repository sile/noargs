noargs
======

[![noargs](https://img.shields.io/crates/v/noargs.svg)](https://crates.io/crates/noargs)
[![Documentation](https://docs.rs/noargs/badge.svg)](https://docs.rs/noargs)
[![Actions Status](https://github.com/sile/noargs/workflows/CI/badge.svg)](https://github.com/sile/noargs/actions)
![License](https://img.shields.io/crates/l/noargs)

`noargs` is an imperative command-line argument parser library with no dependencies, no macros, and no implicit I/O.

Features
--------

- Supports the following argument types:
  - Positional arguments ([`Arg`])
  - Named arguments with values ([`Opt`])
  - Named arguments without values ([`Flag`])
  - Subcommands ([`Cmd`])
- Automatically generates help text
- Simple and minimal interface due to its imperative nature (no complex DSL)
