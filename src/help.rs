use std::collections::HashSet;

use crate::{
    args::{RawArgs, Taken},
    formatter::Formatter,
};

#[derive(Debug)]
pub struct HelpBuilder<'a> {
    args: &'a RawArgs,
    log: Vec<Taken>,
    fmt: Formatter,
    cmd_name: Option<&'static str>,
}

impl<'a> HelpBuilder<'a> {
    pub fn new(args: &'a RawArgs, is_terminal: bool) -> Self {
        let mut this = Self {
            args,
            log: args.log().to_vec(),
            fmt: Formatter::new(is_terminal),
            cmd_name: None,
        };

        // Subcommand handling.
        let Some((name, log_index)) = this.log.iter().enumerate().rev().find_map(|(i, entry)| {
            if let Taken::Cmd(cmd) = entry
                && cmd.present().is_some()
            {
                return Some((cmd.spec().name, i));
            }
            None
        }) else {
            return this;
        };
        this.cmd_name = Some(name);

        let mut log = Vec::new();
        for (i, entry) in this.log.into_iter().enumerate() {
            let mut retain = true;
            if matches!(entry, Taken::Arg(_) | Taken::Cmd(_)) {
                retain = i > log_index;
            }
            if retain {
                log.push(entry);
            }
        }
        this.log = log;

        this
    }

    fn is_full_mode(&self) -> bool {
        self.args.metadata().full_help
    }

    fn doc_lines<'b>(&self, doc: &'b str) -> impl 'b + Iterator<Item = &'b str> {
        let limit = if self.is_full_mode() { usize::MAX } else { 1 };
        doc.lines().take(limit)
    }

    pub fn build(mut self) -> String {
        self.build_description();
        self.build_usage();
        self.build_example();
        self.build_commands();
        self.build_arguments();
        self.build_options();

        let mut text = self.fmt.finish();
        if text.ends_with("\n\n") {
            text.pop();
        }
        text
    }

    fn build_description(&mut self) {
        if self.args.metadata().app_description.is_empty() {
            return;
        }
        for line in self.doc_lines(self.args.metadata().app_description) {
            self.fmt.write(line);
            self.fmt.write("\n");
        }
        self.fmt.write("\n");
    }

    fn build_usage(&mut self) {
        self.fmt.write(&format!(
            "{} {}",
            self.fmt.bold_underline("Usage:"),
            self.fmt.bold(self.args.metadata().app_name),
        ));

        if let Some(name) = self.cmd_name {
            self.fmt.write(&format!(" ... {name}"));
        }

        // Required options.
        for entry in &self.log {
            let Taken::Opt(opt) = entry else {
                continue;
            };
            let opt = opt.spec();
            if opt.example.is_none() {
                continue;
            }
            self.fmt.write(&format!(" --{} <{}>", opt.name, opt.ty));
        }

        // Other options.
        if self.has_options(false) {
            self.fmt.write(" [OPTIONS]");
        }

        // Positional arguments.
        let mut last = None;
        for entry in &self.log {
            let Taken::Arg(arg) = entry else {
                continue;
            };
            let arg = arg.spec();

            if last != Some(arg) {
                self.fmt.write(&format!(" {}", arg.name));
            }
            last = Some(arg);
        }

        // Subcommands.
        if self.has_subcommands() {
            self.fmt.write(" <COMMAND>");
        }

        self.fmt.write("\n\n");
    }

    fn build_example(&mut self) {
        if !self.has_examples() {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Example:\n"));
        self.fmt
            .write(&format!("  $ {}", self.args.metadata().app_name));

        // [NOTE] Need to use `self.args.log()` instead of `self.log` here.
        for entry in self.args.log() {
            if let Some(example) = entry.example() {
                self.fmt.write(&format!(" {}", example));
            }
        }

        self.fmt.write("\n\n");
    }

    fn calc_width_offset_newline<F>(&self, f: F) -> (usize, usize, &'static str)
    where
        F: Fn(&Taken) -> bool,
    {
        if self.is_full_mode() {
            return (0, 4, "\n");
        }
        (
            self.log
                .iter()
                .filter(|e| f(e))
                .map(|e| self.entry_name(e).len())
                .max()
                .unwrap_or_default(),
            1,
            "",
        )
    }

    fn build_commands(&mut self) {
        if !self.has_subcommands() {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Commands:\n"));

        let (width, offset, newline) =
            self.calc_width_offset_newline(|e| matches!(e, Taken::Cmd(_)));
        for entry in &self.log {
            let Taken::Cmd(cmd) = entry else {
                continue;
            };
            let cmd = cmd.spec();

            self.fmt.write(&format!(
                "  {:width$}{newline}",
                self.entry_name(entry),
                width = width
            ));
            for line in cmd.doc.lines() {
                self.fmt
                    .write(&format!("{:offset$}{line}{newline}", "", offset = offset));
            }
            self.fmt.write("\n");
        }
        if !self.is_full_mode() {
            self.fmt.write("\n");
        }
    }

    fn build_arguments(&mut self) {
        if !self.has_positional_args() {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Arguments:\n"));

        let (width, offset, newline) =
            self.calc_width_offset_newline(|e| matches!(e, Taken::Arg(_)));
        let mut known = HashSet::new();
        for entry in &self.log {
            let Taken::Arg(arg) = entry else {
                continue;
            };
            let arg = arg.spec();

            if known.contains(&arg) {
                continue;
            }
            known.insert(arg);

            let name = self.entry_name(entry);
            self.fmt
                .write(&format!("  {:width$}{newline}", name, width = width));

            for line in self.doc_lines(arg.doc) {
                self.fmt
                    .write(&format!("{:offset$}{line}{newline}", "", offset = offset));
            }
            if let Some(default) = arg.default {
                self.fmt.write(&format!(
                    "{:offset$}[default: {default}]{newline}",
                    "",
                    offset = offset
                ));
            }

            self.fmt.write("\n");
        }
        if !self.is_full_mode() {
            self.fmt.write("\n");
        }
    }

    fn entry_name(&self, entry: &Taken) -> String {
        match entry {
            Taken::Opt(opt) => {
                let opt = opt.spec();
                let name = match (opt.short, self.is_full_mode()) {
                    (Some(short), false) => format!("-{short}, --{} <{}>", opt.name, opt.ty),
                    (Some(short), true) => format!("--{}, -{short} <{}>", opt.name, opt.ty),
                    (None, false) => format!("    --{} <{}>", opt.name, opt.ty),
                    (None, true) => format!("--{} <{}>", opt.name, opt.ty),
                };
                self.fmt.bold(&name).into_owned()
            }
            Taken::Flag(flag) => {
                let flag = flag.spec();
                let name = match (flag.short, self.is_full_mode()) {
                    (Some(short), false) => format!("-{short}, --{}", flag.name),
                    (Some(short), true) => format!("--{}, -{short}", flag.name),
                    (None, false) => format!("    --{}", flag.name),
                    (None, true) => format!("--{}", flag.name),
                };
                self.fmt.bold(&name).into_owned()
            }
            Taken::Arg(arg) => {
                format!("{}", self.fmt.bold(arg.spec().name))
            }
            Taken::Cmd(cmd) => self.fmt.bold(cmd.spec().name).into_owned(),
        }
    }

    fn build_options(&mut self) {
        if !self.has_options(true) {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Options:\n"));

        let (width, offset, newline) =
            self.calc_width_offset_newline(|e| matches!(e, Taken::Opt(_) | Taken::Flag(_)));
        let mut known = HashSet::new();
        for entry in &self.log {
            let name = entry.name();
            let (doc, env, default) = match entry {
                Taken::Opt(opt) => {
                    let opt = opt.spec();
                    (opt.doc, opt.env, opt.default)
                }
                Taken::Flag(flag) => {
                    let flag = flag.spec();
                    (flag.doc, flag.env, None)
                }
                _ => continue,
            };

            if known.contains(entry.name()) {
                continue;
            }
            known.insert(name);

            let name = self.entry_name(entry);
            self.fmt
                .write(&format!("  {:width$}{newline}", name, width = width));
            for line in self.doc_lines(doc) {
                self.fmt
                    .write(&format!("{:offset$}{line}{newline}", "", offset = offset));
            }
            if let Some(env) = env {
                self.fmt.write(&format!(
                    "{:offset$}[env: {env}]{newline}",
                    "",
                    offset = offset
                ));
            }
            if let Some(default) = default {
                self.fmt.write(&format!(
                    "{:offset$}[default: {default}]{newline}",
                    "",
                    offset = offset
                ));
            }

            self.fmt.write("\n");
        }
        if !self.is_full_mode() {
            self.fmt.write("\n");
        }
    }

    fn has_positional_args(&self) -> bool {
        self.log.iter().any(|entry| matches!(entry, Taken::Arg(_)))
    }

    fn has_subcommands(&self) -> bool {
        self.log.iter().any(|entry| matches!(entry, Taken::Cmd(_)))
    }

    fn has_options(&self, include_requried: bool) -> bool {
        self.log.iter().any(|entry| match entry {
            Taken::Opt(opt) => include_requried || opt.spec().example.is_none(),
            Taken::Flag(_) => true,
            Taken::Arg(_) | Taken::Cmd(_) => false,
        })
    }

    fn has_examples(&self) -> bool {
        self.log.iter().any(|entry| match entry {
            Taken::Opt(opt) => opt.spec().example.is_some(),
            Taken::Arg(arg) => arg.spec().example.is_some(),
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{HELP_FLAG, VERSION_FLAG};

    use super::*;

    #[test]
    fn flags_help() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "Test command";
        HELP_FLAG.take(&mut args);
        VERSION_FLAG.take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Test command

Usage: <APP_NAME> [OPTIONS]

Options:
  -h, --help    Print help ('--help' for full help, '-h' for summary)
      --version Print version
"#
        );
    }

    #[test]
    fn flags_and_opts_help() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "";
        HELP_FLAG.take(&mut args);
        crate::opt("foo")
            .short('f')
            .doc("An integer\nThis is foo")
            .env("FOO_ENV")
            .default("10")
            .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS]

Options:
  -h, --help        Print help ('--help' for full help, '-h' for summary)
  -f, --foo <VALUE> An integer [env: FOO_ENV] [default: 10]
"#
        );

        args.metadata_mut().full_help = true;
        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS]

Options:
  --help, -h
    Print help ('--help' for full help, '-h' for summary)

  --foo, -f <VALUE>
    An integer
    This is foo
    [env: FOO_ENV]
    [default: 10]
"#
        );
    }

    #[test]
    fn required_opts_help() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "";
        HELP_FLAG.take(&mut args);
        crate::opt("foo")
            .short('f')
            .doc("An integer")
            .example("10")
            .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> --foo <VALUE> [OPTIONS]

Example:
  $ <APP_NAME> --foo 10

Options:
  -h, --help        Print help ('--help' for full help, '-h' for summary)
  -f, --foo <VALUE> An integer
"#
        );
    }

    #[test]
    fn positional_args_help() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "";
        HELP_FLAG.take(&mut args);
        crate::arg("<REQUIRED>")
            .doc("Foo\nDetail is foo")
            .example("3")
            .take(&mut args);
        crate::arg("[OPTIONAL]")
            .doc("Bar")
            .default("9")
            .take(&mut args);
        for _ in 0..3 {
            crate::arg("[MULTI]...").doc("Baz").take(&mut args);
        }

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS] <REQUIRED> [OPTIONAL] [MULTI]...

Example:
  $ <APP_NAME> 3

Arguments:
  <REQUIRED> Foo
  [OPTIONAL] Bar [default: 9]
  [MULTI]... Baz

Options:
  -h, --help Print help ('--help' for full help, '-h' for summary)
"#
        );

        args.metadata_mut().full_help = true;
        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS] <REQUIRED> [OPTIONAL] [MULTI]...

Example:
  $ <APP_NAME> 3

Arguments:
  <REQUIRED>
    Foo
    Detail is foo

  [OPTIONAL]
    Bar
    [default: 9]

  [MULTI]...
    Baz

Options:
  --help, -h
    Print help ('--help' for full help, '-h' for summary)
"#
        );
    }

    #[test]
    fn before_subcommands_help() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "";
        HELP_FLAG.take(&mut args);
        crate::cmd("put").doc("Put an entry").take(&mut args);
        crate::cmd("get").doc("Get an entry").take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS] <COMMAND>

Commands:
  put Put an entry
  get Get an entry

Options:
  -h, --help Print help ('--help' for full help, '-h' for summary)
"#
        );

        args.metadata_mut().full_help = true;
        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> [OPTIONS] <COMMAND>

Commands:
  put
    Put an entry

  get
    Get an entry

Options:
  --help, -h
    Print help ('--help' for full help, '-h' for summary)
"#
        );
    }

    #[test]
    fn after_subcommands_help() {
        let mut args = test_args(&["test", "get"]);
        args.metadata_mut().app_description = "";
        HELP_FLAG.take(&mut args);
        crate::cmd("put").doc("Put an entry").take(&mut args);
        crate::cmd("get").doc("Get an entry").take(&mut args);
        crate::flag("foo").doc("should included").take(&mut args);
        crate::arg("<KEY>")
            .doc("A key string")
            .example("hi")
            .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> ... get [OPTIONS] <KEY>

Example:
  $ <APP_NAME> get hi

Arguments:
  <KEY> A key string

Options:
  -h, --help Print help ('--help' for full help, '-h' for summary)
      --foo  should included
"#
        );

        args.metadata_mut().full_help = true;
        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: <APP_NAME> ... get [OPTIONS] <KEY>

Example:
  $ <APP_NAME> get hi

Arguments:
  <KEY>
    A key string

Options:
  --help, -h
    Print help ('--help' for full help, '-h' for summary)

  --foo
    should included
"#
        );
    }

    #[test]
    fn terminal_formatting() {
        let mut args = test_args(&["test"]);
        crate::flag("help").doc("Print help").take(&mut args);

        // Test that terminal formatting doesn't break the content structure
        let help_terminal = HelpBuilder::new(&args, true).build();
        let help_no_terminal = HelpBuilder::new(&args, false).build();

        // Both should have the same basic structure, just different formatting
        assert!(help_terminal.contains("Usage:"));
        assert!(help_no_terminal.contains("Usage:"));
        assert!(help_terminal.contains("Options:"));
        assert!(help_no_terminal.contains("Options:"));
    }

    #[test]
    fn empty_description() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "";
        crate::flag("help").doc("Print help").take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        // Should not start with empty lines when description is empty
        assert!(help.starts_with("Usage:"));
    }

    #[test]
    fn with_description() {
        let mut args = test_args(&["test"]);
        args.metadata_mut().app_description = "A test application\nWith multiple lines";
        crate::flag("help").doc("Print help").take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        assert!(help.starts_with("A test application"));

        // Test full mode shows all description lines
        args.metadata_mut().full_help = true;
        let help_full = HelpBuilder::new(&args, false).build();
        assert!(help_full.contains("A test application\nWith multiple lines"));
    }

    fn test_args(raw_args: &[&str]) -> RawArgs {
        RawArgs::new(raw_args.iter().map(|a| a.to_string()))
    }
}
