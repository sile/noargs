//! Property-based tests for noargs, driven by noprop.
//!
//! These tests generate command-line inputs and verify the parsing
//! invariants of the imperative `take` API:
//!
//! - A declared flag whose long or short form appears in the args is
//!   detected, even when noise flags are present, and a flag that is
//!   never passed is not detected.
//! - A declared opt's value round-trips through `--k v`, `--k=v`,
//!   `-k v`, and `-kv` forms, a default-declared opt falls back to
//!   `Opt::Default` when absent, and a trailing `--k` / `-k` with no
//!   follow-up produces `Opt::MissingValue` rather than a silent
//!   empty-value success.
//! - A declared positional recovers its value even after flags are
//!   consumed.
//! - Help mode short-circuits the missing / unexpected checks so
//!   `finish()` never errors.
//! - A subcommand is detected only when it is the first unconsumed
//!   token.
//!
//! Names are drawn from fixed pools because `FlagSpec` / `OptSpec` /
//! `ArgSpec` / `CmdSpec` take `&'static str`.

use noargs::{Opt, RawArgs};
use noprop::TestCaseContext;

/// A fixed pool of long/short pairs so we can hand out `&'static str`.
const FLAG_POOL: &[(&str, char)] = &[
    ("alpha", 'a'),
    ("beta", 'b'),
    ("gamma", 'g'),
    ("delta", 'd'),
    ("epsilon", 'e'),
    ("zeta", 'z'),
];

/// A fixed pool of opt long/short pairs.
///
/// The short names avoid the flag shorts so combined forms never alias.
const OPT_POOL: &[(&str, char)] = &[
    ("input", 'i'),
    ("output", 'o'),
    ("endpoint", 'p'),
    ("format", 'f'),
    ("retries", 'r'),
    ("timeout", 't'),
];

/// A fixed pool of subcommand names.
const CMD_POOL: &[&str] = &["start", "stop", "restart", "status", "reload"];

/// Build a `RawArgs` prefixed with a dummy program name (the first arg
/// is always the program name and is skipped by `RawArgs::new`).
fn raw(args: &[String]) -> RawArgs {
    let mut v = vec!["prog".to_string()];
    v.extend_from_slice(args);
    RawArgs::new(v.into_iter())
}

/// Draw a noise count with boundary bias: 0, 1, and MAX each get
/// meaningful probability instead of ~1/MAX.
fn sample_noise_count(ctx: &mut TestCaseContext, max: usize) -> usize {
    noprop::sample_with_boundaries(ctx, &[0usize, 1, max], noprop::Ratio::one_nth(4), |ctx| {
        noprop::sample_usize_in(ctx, 0..=max)
    })
}

/// A declared flag whose long or short form appears in the args must
/// be detected as present, even when other flags are also present.
#[test]
fn declared_flag_is_detected() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_long = Cell::new(false);
    let observed_short = Cell::new(false);
    noprop::Runner::new(0xDEAD_BEEF).run(256, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..FLAG_POOL.len());
        let (name, short) = FLAG_POOL[idx];
        let use_long = noprop::sample_bool(ctx);
        let target = if use_long {
            format!("--{name}")
        } else {
            format!("-{short}")
        };
        // Noise: other flags as long forms only, so their short chars
        // cannot combine into the target's short form.
        let n_noise = sample_noise_count(ctx, 4);
        let mut args_vec: Vec<String> = (0..n_noise)
            .map(|_| {
                let i = noprop::sample_usize_in(ctx, 0..FLAG_POOL.len());
                format!("--{}", FLAG_POOL[i].0)
            })
            .collect();
        args_vec.push(target.clone());
        let mut args = raw(&args_vec);
        let flag = noargs::flag(name).short(short).take(&mut args);
        assert!(
            flag.is_present(),
            "flag {name} (arg {target:?}) must be detected as present among {args_vec:?}"
        );
        if use_long {
            observed_long.set(true);
        } else {
            observed_short.set(true);
        }
        Ok(())
    })?;
    assert!(observed_long.get(), "no case exercised the long form");
    assert!(observed_short.get(), "no case exercised the short form");
    Ok(())
}

/// A flag that is *not* passed on the command line must not be
/// detected as present, regardless of what other flags appear.
///
/// Noise is generated as long forms only: a noise short char could
/// combine with the target's short char (e.g. `-ab`) and legitimately
/// contain the target's short form, which would break the setup.
#[test]
fn undeclared_flag_is_not_detected() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_noise = Cell::new(false);
    noprop::Runner::new(0xBEEF_FACE).run(256, |ctx| {
        let target_idx = noprop::sample_usize_in(ctx, 0..FLAG_POOL.len());
        let (target_name, target_short) = FLAG_POOL[target_idx];
        let n_noise = sample_noise_count(ctx, 4);
        let args_vec: Vec<String> = (0..n_noise)
            .map(|_| {
                let mut i = noprop::sample_usize_in(ctx, 0..FLAG_POOL.len());
                if i == target_idx {
                    i = (i + 1) % FLAG_POOL.len();
                }
                format!("--{}", FLAG_POOL[i].0)
            })
            .collect();
        let mut args = raw(&args_vec);
        let flag = noargs::flag(target_name)
            .short(target_short)
            .take(&mut args);
        assert!(
            !flag.is_present(),
            "flag {target_name} must be absent from {args_vec:?}"
        );
        if n_noise > 0 {
            observed_noise.set(true);
        }
        Ok(())
    })?;
    assert!(observed_noise.get(), "no case exercised noise flags");
    Ok(())
}

/// A declared opt's value must round-trip through both `--k v` and
/// `--k=v` forms, for ordinary, empty, and dash-prefixed values.
#[test]
fn declared_opt_value_round_trips() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_separate = Cell::new(false);
    let observed_equals = Cell::new(false);
    let observed_empty = Cell::new(false);
    let observed_dash = Cell::new(false);
    noprop::Runner::new(0xC0FFEE).run(256, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        // Value classes: ordinary ASCII, empty, dash-prefixed.
        let value_kind = noprop::sample_weighted_index(ctx, &[3, 1, 2]);
        let value = match value_kind {
            0 => format!("v{}", noprop::sample_usize_in(ctx, 0..10_000)),
            1 => String::new(),
            _ => format!("-dash-{}", noprop::sample_usize_in(ctx, 0..1_000)),
        };
        // An empty value is only meaningful in the `--k=` form; in the
        // separate form the empty token would be swallowed as the value.
        let use_equals = value_kind == 1 || noprop::sample_bool(ctx);
        let args_vec = if use_equals {
            vec![format!("--{name}={value}")]
        } else {
            vec![format!("--{name}"), value.clone()]
        };
        let mut args = raw(&args_vec);
        let opt = noargs::opt(name).short(short).take(&mut args);
        assert!(
            matches!(opt, Opt::Long { .. }),
            "expected Opt::Long, got {opt:?}"
        );
        assert_eq!(
            opt.value(),
            value,
            "opt {name} must round-trip its value from {args_vec:?}"
        );
        if use_equals {
            observed_equals.set(true);
        } else {
            observed_separate.set(true);
        }
        match value_kind {
            1 => observed_empty.set(true),
            2 => observed_dash.set(true),
            _ => {}
        }
        Ok(())
    })?;
    assert!(
        observed_separate.get(),
        "no case exercised the `--k v` form"
    );
    assert!(observed_equals.get(), "no case exercised the `--k=v` form");
    assert!(observed_empty.get(), "no case exercised an empty value");
    assert!(observed_dash.get(), "no case exercised a dash value");
    Ok(())
}

/// A declared short opt's value must round-trip through both `-k v`
/// and `-kv` (concatenated) forms, for ordinary, empty, and
/// dash-prefixed values.
#[test]
fn declared_short_opt_value_round_trips() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_separate = Cell::new(false);
    let observed_concatenated = Cell::new(false);
    let observed_empty = Cell::new(false);
    let observed_dash = Cell::new(false);
    noprop::Runner::new(0xCAFE_BABE).run(256, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        // Value classes: ordinary ASCII, empty, dash-prefixed.
        let value_kind = noprop::sample_weighted_index(ctx, &[3, 1, 2]);
        let value = match value_kind {
            0 => format!("v{}", noprop::sample_usize_in(ctx, 0..10_000)),
            1 => String::new(),
            _ => format!("-dash-{}", noprop::sample_usize_in(ctx, 0..1_000)),
        };
        // An empty value is only meaningful in the `-k` + "" separate
        // form; the concatenated form would degrade to a bare `-k` and
        // parse as MissingValue.
        let use_concatenated = value_kind != 1 && noprop::sample_bool(ctx);
        let args_vec = if use_concatenated {
            vec![format!("-{short}{value}")]
        } else {
            vec![format!("-{short}"), value.clone()]
        };
        let mut args = raw(&args_vec);
        let opt = noargs::opt(name).short(short).take(&mut args);
        assert!(
            matches!(opt, Opt::Short { .. }),
            "expected Opt::Short, got {opt:?}"
        );
        assert_eq!(
            opt.value(),
            value,
            "opt {name} must round-trip its value from {args_vec:?}"
        );
        if use_concatenated {
            observed_concatenated.set(true);
        } else {
            observed_separate.set(true);
        }
        match value_kind {
            1 => observed_empty.set(true),
            2 => observed_dash.set(true),
            _ => {}
        }
        Ok(())
    })?;
    assert!(observed_separate.get(), "no case exercised the `-k v` form");
    assert!(
        observed_concatenated.get(),
        "no case exercised the `-kv` form"
    );
    assert!(observed_empty.get(), "no case exercised an empty value");
    assert!(observed_dash.get(), "no case exercised a dash value");
    Ok(())
}

/// A declared opt with a `default(...)` returns that default when the
/// opt is not passed, even when unrelated flags are present.
#[test]
fn declared_opt_default_fallback() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_noise = Cell::new(false);
    noprop::Runner::new(0xFEED_BABE).run(128, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        let n_noise = sample_noise_count(ctx, 4);
        let others: Vec<String> = (0..n_noise)
            .map(|_| {
                let i = noprop::sample_usize_in(ctx, 0..FLAG_POOL.len());
                format!("--{}", FLAG_POOL[i].0)
            })
            .collect();
        let mut args = raw(&others);
        let opt = noargs::opt(name)
            .short(short)
            .default("SENTINEL")
            .take(&mut args);
        assert!(
            matches!(opt, Opt::Default { .. }),
            "expected Opt::Default when opt is absent, got {opt:?}"
        );
        assert_eq!(opt.value(), "SENTINEL");
        if n_noise > 0 {
            observed_noise.set(true);
        }
        Ok(())
    })?;
    assert!(observed_noise.get(), "no case exercised noise flags");
    Ok(())
}

/// A declared required positional recovers its passed value, even
/// when a flag is consumed first.
#[test]
fn declared_positional_round_trips() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_with_flag = Cell::new(false);
    noprop::Runner::new(0xABCD_1234).run(256, |ctx| {
        let n = noprop::sample_usize_in(ctx, 0..10_000);
        let value = format!("pos{n}");
        // Optionally include a leading flag so the positional is not
        // trivially the first token.
        let with_flag = noprop::sample_bool(ctx);
        let mut args_vec = Vec::new();
        if with_flag {
            args_vec.push("--alpha".to_string());
        }
        args_vec.push(value.clone());
        let mut args = raw(&args_vec);
        if with_flag {
            let _ = noargs::flag("alpha").take(&mut args);
        }
        let arg = noargs::arg("<INPUT>").take(&mut args);
        assert_eq!(
            arg.value(),
            value,
            "positional must round-trip from {args_vec:?}"
        );
        if with_flag {
            observed_with_flag.set(true);
        }
        Ok(())
    })?;
    assert!(
        observed_with_flag.get(),
        "no case exercised the flag-prefixed shape"
    );
    Ok(())
}

/// When `help_mode` is `true`, `finish()` never errors even when
/// extra unknown tokens are present — help mode short-circuits the
/// unexpected-arg checks and returns the built help text.
#[test]
fn help_mode_never_errors() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_extra = Cell::new(false);
    noprop::Runner::new(0xFEED_BEEF).run(128, |ctx| {
        let n_extra = sample_noise_count(ctx, 4);
        let extras: Vec<String> = (0..n_extra).map(|i| format!("extra{i}")).collect();
        let mut args = raw(&extras);
        args.metadata_mut().help_mode = true;
        // Declare a required positional we will not provide.
        let _ = noargs::arg("<INPUT>").take(&mut args);
        let finished = args.finish();
        assert!(
            matches!(finished, Ok(Some(_))),
            "finish() in help_mode must return Ok(Some(help)), got {finished:?}"
        );
        if n_extra > 0 {
            observed_extra.set(true);
        }
        Ok(())
    })?;
    assert!(
        observed_extra.get(),
        "no case exercised extra tokens in help mode"
    );
    Ok(())
}

/// Every flag from a distinct subset of the pool that is passed on the
/// command line must be detected as present when declared.
#[test]
fn multiple_declared_flags_are_all_detected() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_multi = Cell::new(false);
    noprop::Runner::new(0xABCDEF01).run(128, |ctx| {
        // Pick a shuffled prefix of the pool.
        let mut chosen: Vec<usize> = (0..FLAG_POOL.len()).collect();
        for i in 0..chosen.len() {
            let swap = noprop::sample_usize_in(ctx, i..chosen.len());
            chosen.swap(i, swap);
        }
        let n = noprop::sample_usize_in(ctx, 1..=FLAG_POOL.len());
        chosen.truncate(n);
        let args_vec: Vec<String> = chosen
            .iter()
            .map(|&i| format!("--{}", FLAG_POOL[i].0))
            .collect();
        let mut args = raw(&args_vec);
        for &i in &chosen {
            let (name, short) = FLAG_POOL[i];
            let flag = noargs::flag(name).short(short).take(&mut args);
            assert!(
                flag.is_present(),
                "flag {name} must be detected among {args_vec:?}"
            );
        }
        if n >= 2 {
            observed_multi.set(true);
        }
        Ok(())
    })?;
    assert!(observed_multi.get(), "no case exercised multiple flags");
    Ok(())
}

/// A `MissingValue` opt reports itself as present but returns an empty
/// value: passing `--opt` as the last token with no follow-up must
/// produce this state, not silently succeed with an empty value.
#[test]
fn opt_without_value_at_end_is_missing_value() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_long = Cell::new(false);
    let observed_short = Cell::new(false);
    noprop::Runner::new(0x0BAD_F00D).run(128, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        let use_long = noprop::sample_bool(ctx);
        let args_vec = if use_long {
            vec![format!("--{name}")]
        } else {
            vec![format!("-{short}")]
        };
        let mut args = raw(&args_vec);
        let opt = noargs::opt(name).short(short).take(&mut args);
        assert!(
            matches!(opt, Opt::MissingValue { long, .. } if long == use_long),
            "trailing `{args_vec:?}` must produce MissingValue, got {opt:?}"
        );
        if use_long {
            observed_long.set(true);
        } else {
            observed_short.set(true);
        }
        Ok(())
    })?;
    assert!(observed_long.get(), "no case exercised the long form");
    assert!(observed_short.get(), "no case exercised the short form");
    Ok(())
}

/// A subcommand is detected only when it is the first unconsumed
/// token: once preceding flags are consumed, a matching token at the
/// head is found, and a non-matching head is not.
#[test]
fn cmd_detected_only_at_first_unconsumed_position() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_present = Cell::new(false);
    let observed_absent = Cell::new(false);
    noprop::Runner::new(0xC0FFEE12).run(128, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..CMD_POOL.len());
        let name = CMD_POOL[idx];
        let n_noise = sample_noise_count(ctx, 4);
        // Preceding flags, consumed before the command check. Pick a
        // shuffled prefix so the flags are unique and every one can be
        // consumed by name.
        let mut order: Vec<usize> = (0..FLAG_POOL.len()).collect();
        for i in 0..order.len() {
            let swap = noprop::sample_usize_in(ctx, i..order.len());
            order.swap(i, swap);
        }
        let chosen = &order[..n_noise];
        let mut args_vec: Vec<String> = chosen
            .iter()
            .map(|&i| format!("--{}", FLAG_POOL[i].0))
            .collect();
        let place_cmd = noprop::sample_bool(ctx);
        let head = if place_cmd {
            name.to_string()
        } else {
            format!("not-a-command-{}", noprop::sample_usize_in(ctx, 0..1_000))
        };
        args_vec.push(head.clone());
        let mut args = raw(&args_vec);
        for &i in chosen {
            let _ = noargs::flag(FLAG_POOL[i].0).take(&mut args);
        }
        let cmd = noargs::cmd(name).take(&mut args);
        assert_eq!(
            cmd.is_present(),
            place_cmd,
            "cmd {name} must be detected iff it is the first unconsumed token: {args_vec:?}"
        );
        if place_cmd {
            observed_present.set(true);
        } else {
            observed_absent.set(true);
        }
        Ok(())
    })?;
    assert!(
        observed_present.get(),
        "no case exercised the present branch"
    );
    assert!(observed_absent.get(), "no case exercised the absent branch");
    Ok(())
}

/// When every token has been consumed by a declared spec, `finish()`
/// must succeed with `Ok(None)` in the normal (non-help) mode.
///
/// The model is the generated token list: flags, `--k=v` opts, and
/// positionals are all declared and taken, so nothing may remain for
/// `finish()` to complain about.
#[test]
fn finish_ok_when_all_args_consumed() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_mixed = Cell::new(false);
    noprop::Runner::new(0x5EED_CAFE).run(128, |ctx| {
        // Shuffled prefixes so the chosen flags / opts are unique.
        let mut flag_order: Vec<usize> = (0..FLAG_POOL.len()).collect();
        for i in 0..flag_order.len() {
            let swap = noprop::sample_usize_in(ctx, i..flag_order.len());
            flag_order.swap(i, swap);
        }
        let mut opt_order: Vec<usize> = (0..OPT_POOL.len()).collect();
        for i in 0..opt_order.len() {
            let swap = noprop::sample_usize_in(ctx, i..opt_order.len());
            opt_order.swap(i, swap);
        }
        let n_flags = noprop::sample_usize_in(ctx, 0..=2);
        let n_opts = noprop::sample_usize_in(ctx, 0..=2);
        let n_pos = noprop::sample_usize_in(ctx, 0..=2);
        // Token list: flags, then opts, then positionals.
        let mut args_vec: Vec<String> = Vec::new();
        for &i in &flag_order[..n_flags] {
            args_vec.push(format!("--{}", FLAG_POOL[i].0));
        }
        for &i in &opt_order[..n_opts] {
            args_vec.push(format!("--{}={i}", OPT_POOL[i].0));
        }
        for i in 0..n_pos {
            args_vec.push(format!("pos{i}"));
        }
        let mut args = raw(&args_vec);
        for &i in &flag_order[..n_flags] {
            let (name, short) = FLAG_POOL[i];
            assert!(
                noargs::flag(name).short(short).take(&mut args).is_present(),
                "flag {name} must be present among {args_vec:?}"
            );
        }
        for &i in &opt_order[..n_opts] {
            let (name, short) = OPT_POOL[i];
            let opt = noargs::opt(name).short(short).take(&mut args);
            assert_eq!(
                opt.value(),
                i.to_string(),
                "opt {name} must be taken from {args_vec:?}"
            );
        }
        for i in 0..n_pos {
            let arg = noargs::arg("<INPUT>").take(&mut args);
            assert_eq!(
                arg.value(),
                format!("pos{i}"),
                "positional {i} must be taken from {args_vec:?}"
            );
        }
        let finished = args.finish();
        assert!(
            matches!(finished, Ok(None)),
            "with all args consumed, finish() must be Ok(None), got {finished:?}"
        );
        if n_flags > 0 && n_opts > 0 && n_pos > 0 {
            observed_mixed.set(true);
        }
        Ok(())
    })?;
    assert!(
        observed_mixed.get(),
        "no case exercised all three token classes"
    );
    Ok(())
}

/// In the normal (non-help) mode, `finish()` must report the first
/// unconsumed token as an `UnexpectedArg` error.
///
/// The oracle is the generated token list: tokens not claimed by a
/// declared spec remain in `RawArgs`, so `finish()` must error; when
/// everything is consumed it must return `Ok(None)`.
#[test]
fn finish_errors_on_unconsumed_arg() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_extra = Cell::new(false);
    let observed_clean = Cell::new(false);
    noprop::Runner::new(0xBAD_C0DE).run(128, |ctx| {
        // Shuffled flag order: the prefix is consumed by declaration,
        // the suffix (if any) remains as unexpected flag tokens.
        let mut flag_order: Vec<usize> = (0..FLAG_POOL.len()).collect();
        for i in 0..flag_order.len() {
            let swap = noprop::sample_usize_in(ctx, i..flag_order.len());
            flag_order.swap(i, swap);
        }
        let n_consume = noprop::sample_usize_in(ctx, 0..=2);
        let mut args_vec: Vec<String> = flag_order[..n_consume]
            .iter()
            .map(|&i| format!("--{}", FLAG_POOL[i].0))
            .collect();
        // Extra tokens: leftover flag names, opt tokens, or bare words.
        // The flag names are drawn from the shuffled suffix so they can
        // never collide with the consumed flags.
        let n_extra = sample_noise_count(ctx, 4);
        let mut suffix_idx = 0;
        for _ in 0..n_extra {
            let kind = noprop::sample_weighted_index(ctx, &[1, 1, 1]);
            let token = match kind {
                0 => {
                    let i = flag_order[n_consume + suffix_idx % flag_order[n_consume..].len()];
                    suffix_idx += 1;
                    format!("--{}", FLAG_POOL[i].0)
                }
                1 => format!(
                    "--{}={}",
                    OPT_POOL[suffix_idx % OPT_POOL.len()].0,
                    suffix_idx
                ),
                _ => format!("word{}", noprop::sample_usize_in(ctx, 0..1_000)),
            };
            suffix_idx += 1;
            args_vec.push(token);
        }
        let mut args = raw(&args_vec);
        for &i in &flag_order[..n_consume] {
            let _ = noargs::flag(FLAG_POOL[i].0).take(&mut args);
        }
        let finished = args.finish();
        if n_extra > 0 {
            assert!(
                matches!(finished, Err(noargs::Error::UnexpectedArg { .. })),
                "unconsumed tokens must produce UnexpectedArg, got {finished:?}"
            );
            observed_extra.set(true);
        } else {
            assert!(
                matches!(finished, Ok(None)),
                "with nothing left, finish() must be Ok(None), got {finished:?}"
            );
            observed_clean.set(true);
        }
        Ok(())
    })?;
    assert!(observed_extra.get(), "no case exercised extra tokens");
    assert!(
        observed_clean.get(),
        "no case exercised a fully consumed finish"
    );
    Ok(())
}

/// Repeated `take`s of the same opt spec consume the matching tokens
/// in order: each `take` yields the first unconsumed match with the
/// correct index, and the spec reports `Opt::None` once exhausted.
///
/// Both the `--k=v` form (immediate return in the parser) and the
/// `--k v` form (pending-value return) are exercised, since they take
/// different code paths in the parser.
#[test]
fn repeated_opt_take_consumes_in_order() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_multi = Cell::new(false);
    let observed_equals = Cell::new(false);
    let observed_separate = Cell::new(false);
    noprop::Runner::new(0x0D_F00D).run(128, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        let m = noprop::sample_with_boundaries(
            ctx,
            &[1usize, 2, 4],
            noprop::Ratio::one_nth(4),
            |ctx| noprop::sample_usize_in(ctx, 1..=4),
        );
        let use_equals = noprop::sample_bool(ctx);
        let args_vec: Vec<String> = if use_equals {
            (0..m).map(|i| format!("--{name}={i}")).collect()
        } else {
            let mut v = Vec::new();
            for i in 0..m {
                v.push(format!("--{name}"));
                v.push(i.to_string());
            }
            v
        };
        let mut args = raw(&args_vec);
        for i in 0..m {
            let opt = noargs::opt(name).short(short).take(&mut args);
            let expected_index = if use_equals {
                i + 1
            } else {
                // prog, then per take: `--k` at 2i+1, value at 2i+2.
                2 * i + 1
            };
            assert!(
                matches!(opt, Opt::Long { index, .. } if index == expected_index),
                "take {i} must consume token at index {expected_index}, got {opt:?}"
            );
            assert_eq!(
                opt.value(),
                i.to_string(),
                "take {i} must recover its own value"
            );
        }
        let exhausted = noargs::opt(name).short(short).take(&mut args);
        assert!(
            matches!(exhausted, Opt::None { .. }),
            "after {m} takes the spec must be exhausted, got {exhausted:?}"
        );
        if m >= 2 {
            observed_multi.set(true);
        }
        if use_equals {
            observed_equals.set(true);
        } else {
            observed_separate.set(true);
        }
        Ok(())
    })?;
    assert!(observed_multi.get(), "no case exercised multiple takes");
    assert!(observed_equals.get(), "no case exercised the `--k=v` form");
    assert!(
        observed_separate.get(),
        "no case exercised the `--k v` form"
    );
    Ok(())
}
