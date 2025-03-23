use std::{collections::HashSet, usize};

use crate::{
    arg::ArgSpec,
    args::{Args, Spec},
    flag::FlagSpec,
    formatter::Formatter,
    opt::OptSpec,
};

#[derive(Debug)]
pub struct HelpBuilder<'a> {
    args: &'a Args,
    specs: Vec<Spec>,
    partial: bool,
    fmt: Formatter,
}

impl<'a> HelpBuilder<'a> {
    pub fn new(args: &'a Args, is_terminal: bool) -> Self {
        let mut this = Self {
            args,
            specs: args.log().to_vec(),
            partial: false,
            fmt: Formatter::new(is_terminal),
        };

        if this.has_subcommands() {
            let old_len = this.specs.len();
            if let Some(level) = this
                .specs
                .iter()
                .map(|spec| spec.min_index())
                .max()
                .unwrap_or_default()
            {
                this.specs.retain(|spec| {
                    (spec.min_index().unwrap_or(0)..=spec.max_index().unwrap_or(usize::MAX))
                        .contains(&level)
                });
            }
            if old_len != this.specs.len() {
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
        for spec in &self.specs {
            let Spec::Opt(spec) = spec else {
                continue;
            };
            if spec.example.is_none() {
                continue;
            }
            self.fmt.write(&format!(" --{} <{}>", spec.name, spec.ty));
        }

        // Other options.
        if self.has_options(false) {
            self.fmt.write(" [OPTIONS]");
        }

        // Positional arguments.
        let mut last = None;
        for &spec in &self.specs {
            let Spec::Arg(spec) = spec else {
                continue;
            };

            if last == Some(spec) {
                if !self.fmt.text().ends_with("...") {
                    self.fmt.write("...");
                }
            } else if spec.example.is_some() {
                // Required argument.
                self.fmt.write(&format!(" <{}>", spec.name));
            } else {
                // Optional argument.
                self.fmt.write(&format!(" [{}]", spec.name));
            }
            last = Some(spec);
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
        for spec in &self.specs {
            match spec {
                Spec::Arg(ArgSpec {
                    example: Some(value),
                    ..
                }) => {
                    self.fmt.write(&format!(" {value}"));
                }
                Spec::Opt(OptSpec {
                    name,
                    example: Some(value),
                    ..
                }) => {
                    self.fmt.write(&format!(" --{name} {value}"));
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

        for &spec in &self.specs {
            let Spec::Subcommand(spec) = spec else {
                continue;
            };

            self.fmt
                .write(&format!("  {}:\n", self.fmt.bold(spec.name)));
            for line in spec.doc.lines() {
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
        for &spec in &self.specs {
            let Spec::Arg(spec) = spec else {
                continue;
            };
            if known.contains(&spec) {
                continue;
            }
            known.insert(spec);

            if spec.example.is_some() {
                self.fmt
                    .write(&format!("  <{}>:\n", self.fmt.bold(spec.name)));
            } else {
                self.fmt
                    .write(&format!("  [{}]:\n", self.fmt.bold(spec.name)));
            }

            for line in spec.doc.lines() {
                self.fmt.write(&format!("    {line}\n"));
            }
            if let Some(default) = spec.default {
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
        let mut last = None;
        // TODO: remove clone
        for spec in self.specs.clone() {
            if Some(spec) == last {
                continue;
            }

            match spec {
                Spec::Opt(spec) => self.build_opt(spec),
                Spec::Flag(spec) => self.build_flag(spec),
                _ => {}
            }
            last = Some(spec);
        }
    }

    fn build_opt(&mut self, spec: OptSpec) {
        let names = if let Some(short) = spec.short {
            format!("--{}, -{short}", spec.name)
        } else {
            format!("--{}", spec.name)
        };
        self.fmt
            .write(&format!("  {} <{}>:\n", self.fmt.bold(&names), spec.ty));
        for line in spec.doc.lines() {
            self.fmt.write(&format!("    {line}\n"));
        }
        if let Some(env) = spec.env {
            self.fmt.write(&format!("    [env: {env}]\n"));
        }
        if let Some(default) = spec.default {
            self.fmt.write(&format!("    [default: {default}]\n"));
        }
    }

    fn build_flag(&mut self, spec: FlagSpec) {
        let names = if let Some(short) = spec.short {
            format!("--{}, -{short}:", spec.name)
        } else {
            format!("--{}:", spec.name)
        };
        self.fmt.write(&format!("  {}\n", self.fmt.bold(&names)));
        for line in spec.doc.lines() {
            self.fmt.write(&format!("    {line}\n"));
        }
        if let Some(env) = spec.env {
            self.fmt.write(&format!("    [env: {env}]\n"));
        }
    }

    fn has_positional_args(&self) -> bool {
        self.specs.iter().any(|spec| matches!(spec, Spec::Arg(_)))
    }

    fn has_subcommands(&self) -> bool {
        self.specs
            .iter()
            .any(|spec| matches!(spec, Spec::Subcommand(_)))
    }

    fn has_options(&self, include_requried: bool) -> bool {
        self.specs.iter().any(|spec| match spec {
            Spec::Opt(spec) => include_requried || spec.example.is_none(),
            Spec::Flag(_) => true,
            Spec::Arg(_) | Spec::Subcommand(_) => false,
        })
    }

    fn has_examples(&self) -> bool {
        self.specs.iter().any(|spec| match spec {
            Spec::Opt(spec) => spec.example.is_some(),
            Spec::Arg(spec) => spec.example.is_some(),
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::subcommand::SubcommandSpec;

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
