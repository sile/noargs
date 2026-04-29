---
name: noargs
description: >
  Work with the noargs Rust crate: an imperative, no-dependency, no-macro,
  no-implicit-I/O command-line parser. Build CLIs with positional arguments
  (`arg`), named options with values (`opt`), named flags (`flag`), and
  subcommands (`cmd`); drive help / version via `HELP_FLAG` / `VERSION_FLAG`;
  finalize with `RawArgs::finish()`; convert values via `.then(...)` /
  `.present_and_then(...)`; collect repeated options or positional arrays with
  a `while let` loop. Use when the task mentions noargs, an imperative CLI
  parser, or migrating off clap/argh/structopt to a macro-free alternative.
license: MIT
compatibility: Requires Rust 1.88+ and cargo.
---

# noargs

Use this skill when integrating `noargs` into Rust code. Focus on the crate's
actual API shape and usage patterns, not generic CLI-parsing background.

## What this crate exposes

- `noargs::raw_args()` — convenience wrapper around `RawArgs::new(std::env::args())`
- `noargs::arg(name)` → `ArgSpec` — positional argument
- `noargs::opt(name)` → `OptSpec` — named argument with value (`--name VALUE`)
- `noargs::flag(name)` → `FlagSpec` — named argument without value (`--name`)
- `noargs::cmd(name)` → `CmdSpec` — subcommand
- `noargs::HELP_FLAG` — pre-built `--help, -h` flag spec
- `noargs::VERSION_FLAG` — pre-built `--version` flag spec
- `noargs::Result<T>` and `noargs::Error`
- Runtime types — each an enum whose variants record how the value was
  supplied:
  - `Arg`: `Positional` / `Default` / `Example` / `None`
  - `Opt`: `Long` / `Short` / `Env` / `Default` / `Example` / `MissingValue` / `None`
  - `Flag`: `Long` / `Short` / `Env` / `None`
  - `Cmd`: `Some` / `None`

## The imperative parsing loop

The crate is built around one pattern — there is no declarative schema:

1. Build `RawArgs` from argv (`noargs::raw_args()`).
2. Set help metadata (`args.metadata_mut().app_name = …`, `app_description = …`).
3. Consume `VERSION_FLAG` first and exit if present.
4. Consume `HELP_FLAG` with `take_help` — this sets `help_mode` / `full_help`
   in metadata so subsequent `take` calls yield example/default values rather
   than the real input.
5. Call `.take(&mut args)` on each spec to consume matching tokens from the
   buffer. `.take()` mutates `RawArgs`; tokens are removed as they are claimed.
6. Call `args.finish()?` at the end — in help mode it returns
   `Ok(Some(help_text))`; otherwise it errors on unclaimed tokens or missing
   subcommand.

## Choosing the right API

1. **Fixed positional** — `arg("<NAME>")` for required, `arg("[NAME]")` for
   optional. Use `<>` / `[]` purely as a naming convention; they do not
   drive behavior — required-ness comes from calling `.then()` (required) vs
   `.present_and_then()` (optional) or setting `.default()`.
2. **Named option with value** — `opt("long-name").short('f').ty("VALUE")`.
   Short-only options are not supported; every option must have a long name.
3. **Boolean flag** — `flag("long-name").short('f')`. `.is_present()` is the
   standard reader. Like `opt`, short-only flags are not supported.
4. **Subcommand** — `cmd("name")`. Only matches if it's the *next unconsumed*
   token. Consume global flags/opts before `cmd().take()`, or the command
   token will be shadowed by leading flag tokens.
5. **Environment fallback** — `opt(...).env("VAR")` for options,
   `flag(...).env("VAR")` for flags. Empty-string values are ignored.
6. **Required-in-help** — add `.example("…")` to any required option /
   positional so help-mode output has something to print. Optional fields with
   `.default()` or `.present_and_then()` do not need `.example()`.
   `.example()` is consulted *only* when `help_mode` is on; it never affects
   parsing of real argv. Required-ness is decided by `.then()` (required) vs
   `.present_and_then()` (optional) or `.default()`.

## Usage gotchas

- **Order matters.** Call `flag()` / `opt()` `.take()` before `arg()`
  `.take()`. `arg()` eagerly consumes the first remaining token —
  including a leftover `-v` (declared flag taken too late) or `--bogus`
  (typo / undeclared flag). `args.finish()` reports unconsumed tokens as
  `Error::UnexpectedArg`, but a typo that fits an empty positional slot
  is bound silently — e.g. `prog --format json --bogus` leaves `--bogus`
  as the `<INPUT>` value with no error.
- **`take_help` vs `take`.** `HELP_FLAG.take_help(&mut args)` flips
  `metadata.help_mode = true` when `-h`/`--help` is present, and also sets
  `full_help = true` for `--help` (long form). Use `take_help`, not plain
  `take`, for the help flag.
- **Help mode collapses `take`.** Once `help_mode` is on, `ArgSpec::take` /
  `OptSpec::take` return `Default`/`Example`/`None` variants without scanning
  argv. You still need to call each `.take()` so it appears in the help log;
  but after `args.finish()?` returns `Some(help)`, print it and return early.
  Guard application logic behind `if args.metadata().help_mode { return … }`
  inside subcommand handlers that early-return before `finish()`.
- **`.then(f)` is "required".** It returns `Err(Error::MissingArg/MissingOpt)`
  when the value isn't present, so use it for required args or for required
  parsing after `.default()`. Use `.present_and_then(f)` for optional values
  — it returns `Ok(None)` when absent.
- **Repeated same-name options / positional arrays.** Call `.take()` in a
  `while let Some(v) = spec.take(&mut args).present_and_then(…)? { … }` loop.
  Each iteration consumes one match.
- **Option value forms.** Long: `--name=value` or `--name value`. Short:
  `-fvalue` (concatenated) or `-f value` (separate). A bare `-f` with no
  following token yields `Opt::MissingValue` — `.then()` surfaces this as
  `Error::MissingOpt`.
- **Short flag packing.** `-abc` expands to flags `a`, `b`, `c` when every
  char passes `metadata.is_valid_flag_chars` (default: ASCII alphabetic). If
  an app uses short options whose values may be alphabetic (e.g. `-khello`),
  tighten `is_valid_flag_chars` to an allow-list of actual flag letters.
- **`Error` is terminal-only.** It deliberately does *not* implement
  `std::error::Error` or `Display`; only `Debug`. Return it from `main` via
  `noargs::Result<()>`. Any `Display`-able error converts into `Error` via the
  blanket `From` impl, so `?` works with `parse()`, `io::Error`, etc.
- **`cmd` stops at the first unconsumed token.** If flags haven't been
  consumed yet, a subcommand that sits after them won't be found. Consume
  global `flag`/`opt` first, then `cmd`.
- **`default("…")` takes `&'static str`.** For runtime-computed defaults, use
  a `static LazyLock<String>` and pass `&*LAZY`.

## API patterns to preserve

Minimal top-level CLI:

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // flags / opts BEFORE positional args
    let verbose = noargs::flag("verbose").short('v').take(&mut args).is_present();
    let retries: usize = noargs::opt("retries")
        .short('r').ty("N").default("1")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    let format: String = noargs::opt("format")
        .ty("FORMAT").example("json")            // required → needs example()
        .take(&mut args)
        .then(|o| o.value().parse())?;
    let input: String = noargs::arg("<INPUT>")
        .example("input.txt")                    // required → needs example()
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let output: Option<String> = noargs::arg("[OUTPUT]")
        .take(&mut args)
        .present_and_then(|a| a.value().parse())?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    // application logic using verbose / retries / format / input / output
    let _ = (verbose, retries, format, input, output);
    Ok(())
}
```

Repeated same-name option and positional array:

```rust
let include_spec = noargs::opt("include").short('I').ty("PATH");
let mut includes = Vec::<String>::new();
while let Some(path) = include_spec
    .take(&mut args)
    .present_and_then(|o| o.value().parse())?
{
    includes.push(path);
}

let first: String = noargs::arg("<INPUT>")
    .example("a.txt")
    .take(&mut args)
    .then(|a| a.value().parse())?;
let rest_spec = noargs::arg("[INPUT]...");
let mut inputs = vec![first];
while let Some(v) = rest_spec
    .take(&mut args)
    .present_and_then(|a| a.value().parse())?
{
    inputs.push(v);
}
```

Subcommand routing with per-command scope:

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    let _ = try_run_hello(&mut args)? || try_run_sum(&mut args)?;
    if let Some(help) = args.finish()? {
        print!("{help}");
    }
    Ok(())
}

fn try_run_hello(args: &mut noargs::RawArgs) -> noargs::Result<bool> {
    if !noargs::cmd("hello").doc("Print a greeting").take(args).is_present() {
        return Ok(false);
    }
    let loud = noargs::flag("loud").short('l').take(args).is_present();
    let name: String = noargs::arg("<NAME>")
        .example("Alice")
        .take(args)
        .then(|a| a.value().parse())?;

    // IMPORTANT: skip real work while building help text.
    if args.metadata().help_mode { return Ok(true); }

    let msg = format!("Hello, {name}!");
    println!("{}", if loud { msg.to_uppercase() } else { msg });
    Ok(true)
}
```

Custom validation via `.then`:

```rust
let port: u16 = noargs::opt("port").short('p').default("8080")
    .take(&mut args)
    .then(|o| -> Result<_, Box<dyn std::error::Error>> {
        let n: u16 = o.value().parse()?;
        if n == 0 { return Err("port must be > 0".into()); }
        Ok(n)
    })?;
```

Environment-variable fallbacks:

```rust
let endpoint: String = noargs::opt("endpoint")
    .env("MYAPP_ENDPOINT")
    .default("http://localhost:8080")
    .take(&mut args)
    .then(|o| o.value().parse())?;

let debug = noargs::flag("debug").env("MYAPP_DEBUG").take(&mut args).is_present();
```

## Runnable references

The repo's `examples/` directory contains runnable templates that demonstrate
complete patterns — read them end-to-end when a snippet isn't enough:

- `examples/basics.rs` — single-command CLI with flags, options, env
  fallbacks, required + optional positionals, and a `--dry-run` path.
  Comments flag the common order / `example()` / help-mode pitfalls.
- `examples/arrays.rs` — repeated `opt` and positional-array patterns using
  `while let Some(v) = spec.take(...).present_and_then(...)? { ... }`, plus
  the `<NAME>` / `[NAME]...` naming convention.
- `examples/subcommands.rs` — `try_run_*` command routing pattern (bool
  short-circuit chain), per-command scoped flags/args, and the
  `if args.metadata().help_mode { return Ok(true); }` guard inside handlers.

## Practical hints

- Start by copying the structure of `examples/basics.rs` or
  `examples/subcommands.rs` instead of re-deriving the order-sensitive steps
  (metadata → version → help → opts/flags → args → `finish`).
- For any required field, set `.example("…")` so `--help` output is not blank.
- For any optional field, prefer `.default("…")` + `.then()` when a sensible
  default exists; reach for `.present_and_then()` + `Option<T>` only when the
  field really is absent-vs-present.
- Don't try to match options with only a short name — the crate doesn't
  support it. Give every `opt` / `flag` a long name, with `.short('x')`
  optional.
- When `finish()` reports `UndefinedCommand` unexpectedly, the real cause is
  usually a leading flag/opt not yet consumed — consume global flags first,
  then call `cmd(...).take()`.
- The crate is std-only (uses `std::env`, `std::io::IsTerminal`); it does not
  target `no_std`.
- Values are parsed with `str::parse` through `.then()` — any `FromStr` type
  works, and any `Display` error is turned into `Error::InvalidArg/InvalidOpt`
  with a helpful message.
