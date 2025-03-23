use std::{borrow::Cow, collections::HashSet, usize};

use crate::{
    args::{Args, Taken},
    formatter::Formatter,
};

#[derive(Debug)]
pub struct HelpBuilder<'a> {
    args: &'a Args,
    log: Vec<Taken>,
    partial: bool,
    fmt: Formatter,
}

impl<'a> HelpBuilder<'a> {
    pub fn new(args: &'a Args, is_terminal: bool) -> Self {
        let mut this = Self {
            args,
            log: args.log().to_vec(),
            partial: false,
            fmt: Formatter::new(is_terminal),
        };

        if this.has_subcommands() {
            let old_len = this.log.len();
            if let Some(level) = this
                .log
                .iter()
                .map(|spec| spec.min_index())
                .max()
                .unwrap_or_default()
            {
                this.log.retain(|spec| {
                    (spec.min_index().unwrap_or(0)..=spec.max_index().unwrap_or(usize::MAX))
                        .contains(&level)
                });
            }
            if old_len != this.log.len() {
                this.partial = true;
            }
        }

        this
    }

    pub fn build(mut self) -> String {
        self.build_description();
        self.build_usage();
        self.build_example();
        self.build_commands();
        self.build_arguments();
        self.build_options();
        self.fmt.finish()
    }

    fn build_description(&mut self) {
        if self.args.metadata().app_description.is_empty() {
            return;
        }
        self.fmt.write(self.args.metadata().app_description);
        self.fmt.write("\n\n");
    }

    fn build_usage(&mut self) {
        self.fmt.write(&format!(
            "{} {}",
            self.fmt.bold_underline("Usage:"),
            self.fmt.bold(self.args.metadata().app_name),
        ));

        if self.partial {
            self.fmt.write(" ...");
        }

        // TODO: subcommand

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

            if last == Some(arg) {
                if !self.fmt.text().ends_with("...") {
                    self.fmt.write("...");
                }
            } else if arg.example.is_some() {
                // Required argument.
                self.fmt.write(&format!(" <{}>", arg.name));
            } else {
                // Optional argument.
                self.fmt.write(&format!(" [{}]", arg.name));
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
        for entry in &self.log {
            match entry {
                Taken::Arg(arg) => {
                    if let Some(value) = arg.spec().example {
                        self.fmt.write(&format!(" {value}"));
                    }
                }
                Taken::Opt(opt) => {
                    if let Some(value) = opt.spec().example {
                        self.fmt.write(&format!(" --{} {value}", opt.spec().name));
                    }
                }
                _ => {}
            }
        }
        self.fmt.write("\n\n");
    }

    fn build_commands(&mut self) {
        if !self.has_subcommands() {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Commands:\n"));

        for entry in &self.log {
            let Taken::Subcommand(cmd) = entry else {
                continue;
            };
            let cmd = cmd.spec();

            self.fmt.write(&format!("  {}:\n", self.fmt.bold(cmd.name)));
            for line in cmd.doc.lines() {
                self.fmt.write(&format!("    {line}\n"));
            }
        }

        self.fmt.write("\n");
    }

    fn build_arguments(&mut self) {
        if !self.has_positional_args() {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Arguments:\n"));

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

            if arg.example.is_some() {
                self.fmt
                    .write(&format!("  <{}>:\n", self.fmt.bold(arg.name)));
            } else {
                self.fmt
                    .write(&format!("  [{}]:\n", self.fmt.bold(arg.name)));
            }

            for line in arg.doc.lines() {
                self.fmt.write(&format!("    {line}\n"));
            }
            if let Some(default) = arg.default {
                self.fmt.write(&format!("    [default: {default}]\n"));
            }
        }

        self.fmt.write("\n");
    }

    fn build_options(&mut self) {
        if !self.has_options(true) {
            return;
        }

        self.fmt.write(&self.fmt.bold_underline("Options:\n"));

        let mut known = HashSet::new();
        for entry in &self.log {
            let name = entry.name();
            let (short, ty, doc, env, default) = match entry {
                Taken::Opt(opt) => {
                    let opt = opt.spec();
                    (opt.short, Some(opt.ty), opt.doc, opt.env, opt.default)
                }
                Taken::Flag(flag) => {
                    let flag = flag.spec();
                    (flag.short, None, flag.doc, flag.env, None)
                }
                _ => continue,
            };

            if known.contains(entry.name()) {
                continue;
            }
            known.insert(name);

            let names = if let Some(short) = short {
                format!("--{name}, -{short}")
            } else {
                format!("--{name}")
            };
            self.fmt.write(&format!(
                "  {}{}:\n",
                self.fmt.bold(&names),
                if let Some(ty) = ty {
                    Cow::Owned(format!(" <{ty}>"))
                } else {
                    Cow::Borrowed("")
                }
            ));
            for line in doc.lines() {
                self.fmt.write(&format!("    {line}\n"));
            }
            if let Some(env) = env {
                self.fmt.write(&format!("    [env: {env}]\n"));
            }
            if let Some(default) = default {
                self.fmt.write(&format!("    [default: {default}]\n"));
            }
        }
    }

    fn has_positional_args(&self) -> bool {
        self.log.iter().any(|entry| matches!(entry, Taken::Arg(_)))
    }

    fn has_subcommands(&self) -> bool {
        self.log
            .iter()
            .any(|entry| matches!(entry, Taken::Subcommand(_)))
    }

    fn has_options(&self, include_requried: bool) -> bool {
        self.log.iter().any(|entry| match entry {
            Taken::Opt(opt) => include_requried || opt.spec().example.is_none(),
            Taken::Flag(_) => true,
            Taken::Arg(_) | Taken::Subcommand(_) => false,
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
    use crate::{arg::ArgSpec, flag::FlagSpec, opt::OptSpec, subcommand::SubcommandSpec};

    use super::*;

    #[test]
    fn flags_help() {
        let mut args = args(&["noargs"]);
        args.metadata_mut().app_description = "Test command";
        FlagSpec::HELP.take(&mut args);
        FlagSpec::VERSION.take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Test command

Usage: noargs [OPTIONS]

Options:
  --help, -h:
    Print help
  --version:
    Print version
"#
        );
    }

    #[test]
    fn flags_and_opts_help() {
        let mut args = args(&["noargs"]);
        args.metadata_mut().app_description = "";
        FlagSpec::HELP.take(&mut args);
        OptSpec {
            name: "foo",
            short: Some('f'),
            ty: "INTEGER",
            doc: "An integer\nThis is foo",
            env: Some("FOO_ENV"),
            default: Some("10"),
            ..Default::default()
        }
        .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: noargs [OPTIONS]

Options:
  --help, -h:
    Print help
  --foo, -f <INTEGER>:
    An integer
    This is foo
    [env: FOO_ENV]
    [default: 10]
"#
        );
    }

    #[test]
    fn required_opts_help() {
        let mut args = args(&["noargs"]);
        args.metadata_mut().app_description = "";
        FlagSpec::HELP.take(&mut args);
        OptSpec {
            name: "foo",
            short: Some('f'),
            ty: "INTEGER",
            doc: "An integer",
            example: Some("10"),
            ..Default::default()
        }
        .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: noargs --foo <INTEGER> [OPTIONS]

Example:
  $ noargs --foo 10

Options:
  --help, -h:
    Print help
  --foo, -f <INTEGER>:
    An integer
"#
        );
    }

    #[test]
    fn positional_args_help() {
        let mut args = args(&["noargs"]);
        args.metadata_mut().app_description = "";
        FlagSpec::HELP.take(&mut args);
        ArgSpec {
            name: "REQUIRED",
            doc: "Foo",
            example: Some("3"),
            ..Default::default()
        }
        .take(&mut args);
        ArgSpec {
            name: "OPTIONAL",
            doc: "Bar",
            default: Some("9"),
            ..Default::default()
        }
        .take(&mut args);
        for _ in 0..3 {
            ArgSpec {
                name: "MULTI",
                doc: "Baz",
                ..Default::default()
            }
            .take(&mut args);
        }

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: noargs [OPTIONS] <REQUIRED> [OPTIONAL] [MULTI]...

Example:
  $ noargs 3

Arguments:
  <REQUIRED>:
    Foo
  [OPTIONAL]:
    Bar
    [default: 9]
  [MULTI]:
    Baz

Options:
  --help, -h:
    Print help
"#
        );
    }

    #[test]
    fn before_subcommands_help() {
        let mut args = args(&["noargs"]);
        args.metadata_mut().app_description = "";
        FlagSpec::HELP.take(&mut args);
        SubcommandSpec {
            name: "put",
            doc: "Put an entry",
            ..Default::default()
        }
        .take(&mut args);
        SubcommandSpec {
            name: "get",
            doc: "Get an entry",
            ..Default::default()
        }
        .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: noargs [OPTIONS] <COMMAND>

Commands:
  put:
    Put an entry
  get:
    Get an entry

Options:
  --help, -h:
    Print help
"#
        );
    }

    #[test]
    fn after_subcommands_help() {
        let mut args = args(&["noargs", "get"]);
        args.metadata_mut().app_description = "";
        FlagSpec::HELP.take(&mut args);
        SubcommandSpec {
            name: "put",
            doc: "Put an entry",
            ..Default::default()
        }
        .take(&mut args);
        let cmd = SubcommandSpec {
            name: "get",
            doc: "Get an entry",
            ..Default::default()
        }
        .take(&mut args);
        ArgSpec {
            name: "KEY",
            example: Some("foo"),
            min_index: cmd.index(),
            ..Default::default()
        }
        .take(&mut args);

        let help = HelpBuilder::new(&args, false).build();
        println!("{help}");
        assert_eq!(
            help,
            r#"Usage: noargs [OPTIONS] <COMMAND>

Commands:
  put:
    Put an entry
  get:
    Get an entry

Options:
  --help, -h:
    Print help
"#
        );
    }

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }
}
