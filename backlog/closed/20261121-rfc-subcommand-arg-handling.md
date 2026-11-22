# RFC: Hand `RawArgs` to the subcommand, and check inside the handler

## Summary

A subcommand handler should take `RawArgs` by value, run every `take` it needs, and
call `finish` itself -- for the help text and for the argument checks -- before
it does any of its work. This settles where `finish` is called when there are
several subcommands, and it lets a bad command line be reported before the
command acts. The library keeps its signatures: `finish` still consumes
`self`, and nothing in `src/` changes. What changes is how subcommands are
dispatched and handled, shown in `examples/subcommands.rs`.

This answers the feedback in
[feedback-231009](../closed/20261121-feedback-231009.md).

## Motivation

`finish(self)` consumes `self`, so it can be called once, at the end. With
subcommands, "the end" is in `main`, after every handler has returned. A
handler therefore cannot check its own arguments, and the whole command line is
checked only after the command has run.

Concretely, `tool create NAME EXTRA` reports `EXTRA` as unconsumed, exits 1, and
does so after the file for `NAME` has been written. A command that moves files
fares the same way: a mistyped command line changes the file system and then
fails. The command line was wrong the whole time, yet the change happened first.

There is a second cost today. Because `finish` is called in `main`, the help text
can only be produced there too, and a handler that does its work before returning
runs while `help_mode` is set. Every handler therefore carries the same guard:

```rust
if args.metadata().help_mode {
    return Ok(true);
}
```

Nothing enforces it. A handler that leaves it out prints to stdout while the
user asked for help, and the tests do not catch it. Moving the `finish` call into
the handler turns this guard into the call that already has to be there.

## Design

`main` dispatches on the subcommand and hands `RawArgs` over by value:

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

    if noargs::cmd("hello").doc("Print a greeting").take(&mut args).is_present() {
        return run_hello(args);
    }
    if noargs::cmd("sum").doc("Add two integers").take(&mut args).is_present() {
        return run_sum(args);
    }
    if noargs::cmd("echo").doc("Print a message").take(&mut args).is_present() {
        return run_echo(args);
    }

    // No subcommand: top-level help, and the top-level argument checks.
    if let Some(help) = args.finish()? {
        print!("{help}");
    }
    Ok(())
}
```

Each handler takes `RawArgs` by value, does its `take`s, calls `finish`, and only
then does its work:

```rust
fn run_hello(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let loud = noargs::flag("loud")
        .short('l')
        .doc("Print greeting in upper case")
        .take(&mut args)
        .is_present();
    let name: String = noargs::arg("<NAME>")
        .doc("Name to greet")
        .example("Alice")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    // Help text for `hello`, and the `hello` argument checks. Both happen here.
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let message = format!("Hello, {name}!");
    if loud {
        println!("{}", message.to_uppercase());
    } else {
        println!("{message}");
    }
    Ok(())
}
```

The shape has three properties worth naming:

- `finish` is called exactly once on every path. `main` owns the instance until
  it dispatches; afterwards the handler owns it. Ownership makes the count a
  compile-time fact rather than a convention.
- `finish` does the two jobs that were separate before. `Some(help)` means the
  user asked for help, and `None` means the arguments were checked and are fine.
  The old `help_mode` guard and the old end-of-`main` check become one call.
- Every side effect sits below the `finish` call, so a stray argument or a
  missing required argument is reported before anything is written or moved.

No signature in `src/` changes. `RawArgs::finish` still takes `self`, `take`
still takes `&mut RawArgs`, and the `Taken` accessors are untouched. This RFC is
about the order the calls are made in, which the library already permits, and
`examples/subcommands.rs` is where that order is written down.

One consequence of `RawArgs` moving into the handler is that the dispatch arms
leave with `return`, since `args` is gone once it has been moved. The example
above is written that way.

## Alternatives

Make `finish` take `&self` instead. The handler would keep `&mut RawArgs` and
call `args.finish()?` without giving the instance up. This reaches the same
result for this RFC -- the checks still run before the side effects -- and it is
a smaller diff, since it leaves `examples/` and `src/` alone. Widening the
signature is where it stops being acceptable: after the change, `take`,
`finish`, `take` would compile.

That sequence must not compile, and the reason is semantic, not stylistic.
`finish` is not a query but a state transition: it decides what the command line
meant. `Error::check_unexpected_arg` judges the buffer as it stands, so a `take`
after `finish` would make a token that was reported as unexpected become a
legitimate argument later, and the same instance would answer "invalid" and
"valid" at different moments. In help mode it is worse: `finish` returns
`Ok(Some(help))` and the `take`s in that mode yield `Default`/`Example` without
reading argv, so a `take` after it would succeed on a value nobody supplied. The
`Taken` log turns into "the last value read for this spec" rather than the record
of how the argument was actually given, and which entry is true depends on the
call order. In one sentence: after `finish`, the meaning of the buffer is
settled, and reading it again asks a question whose answer no longer means
anything.

Leaving that to a written rule -- "do not `take` after `finish`" -- would work,
but it replaces a fact the compiler checks with a convention every caller has to
know, and a convention is exactly what a new handler forgets. Keep `finish(self)`.
The narrow form (`&self`) is not a change to be made when something needs it; it
is the form that lets the forbidden sequence back in, so it is not the answer
even then. Its remaining merit -- that it does not remove the per-handler
`help_mode` guard -- is the other half of the motivation, so it does not rescue
the alternative either.

Add a `validate(&self)` that runs the checks without consuming `self`, and leave
`finish` as the end-of-parsing call. Two entry points mean two places to
remember and a question the design does not answer today: what `validate`
returns in help mode. `noargs` is small on purpose, and an extra public method is
the largest thing this library can gain.

Give subcommands a scope, so the checks run when the scope ends (a closure-based
`args.subcommand(|args| ...)`). This enforces the order structurally, but it adds
the DSL shape that `noargs` exists to avoid: the handler would become a closure
that borrows and returns, and the dispatch in `main` would stop being ordinary
`if`s. It is the most machinery for the smallest remaining problem.

## Open questions

- Is `run_*(mut args: RawArgs)` the right signature, or should handlers take
  `&mut RawArgs` and the dispatch keep the instance? The value form is what lets
  `finish(self)` be called inside the handler, so it is what this RFC needs; the
  reference form only works with the `&self` alternative above.
- Are there handlers that need to `take` again after `finish`? Nothing in this
  library does today, and the value form makes it a compile error rather than a
  silent mistake. The answer if one turns up is that the handler is wrong, not
  that the signature is: what it needs to read after `finish` is decided before
  `finish`, so the `take` belongs above the call. This is why the `&self`
  alternative is rejected outright rather than postponed -- it would answer this
  question by removing the constraint that makes the ordering meaningful.
- Should `RawArgs` become `Clone` so a handler can be given a copy? There is no
  plan to do so, and the value form is what makes that unnecessary.

## Outcome

Accepted, and implemented by PR #25, which is merged: the subcommand handlers in
`examples/subcommands.rs` now take `RawArgs` by value and call `finish`
themselves, after their `take`s and before any side effect, and `README.md` and
`skills/noargs/SKILL.md` follow the same shape. `src/` is unchanged, including
`RawArgs::finish(self)`, as the design asked.

One point came out sharper than the text above has it. The rejection of
`finish(&self)` is not about keeping an option open: a `take` after `finish`
breaks what `finish` means. It settles what the command line meant, so a later
`take` would either revive a token already reported as unexpected or, in help
mode, succeed on a value nobody supplied, and the `Taken` log would become "the
last value read" instead of the record of how the argument was given. The open
question about handlers that want to read after `finish` is answered the same
way: such a handler is reading in the wrong place, and the read belongs above
the call. `skills/noargs/SKILL.md` states the rule and that reason.
