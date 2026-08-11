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
/// `--k=v` forms.
#[test]
fn declared_opt_value_round_trips() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_separate = Cell::new(false);
    let observed_equals = Cell::new(false);
    noprop::Runner::new(0xC0FFEE).run(256, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        // A simple ASCII value that will not be parsed as a flag/opt.
        let n = noprop::sample_usize_in(ctx, 0..10_000);
        let value = format!("v{n}");
        let use_equals = noprop::sample_bool(ctx);
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
        Ok(())
    })?;
    assert!(
        observed_separate.get(),
        "no case exercised the `--k v` form"
    );
    assert!(observed_equals.get(), "no case exercised the `--k=v` form");
    Ok(())
}

/// A declared short opt's value must round-trip through both `-k v`
/// and `-kv` (concatenated) forms.
#[test]
fn declared_short_opt_value_round_trips() -> noprop::TestResult {
    use std::cell::Cell;
    let observed_separate = Cell::new(false);
    let observed_concatenated = Cell::new(false);
    noprop::Runner::new(0xCAFE_BABE).run(256, |ctx| {
        let idx = noprop::sample_usize_in(ctx, 0..OPT_POOL.len());
        let (name, short) = OPT_POOL[idx];
        let n = noprop::sample_usize_in(ctx, 0..10_000);
        let value = format!("v{n}");
        let use_concatenated = noprop::sample_bool(ctx);
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
        Ok(())
    })?;
    assert!(observed_separate.get(), "no case exercised the `-k v` form");
    assert!(
        observed_concatenated.get(),
        "no case exercised the `-kv` form"
    );
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
