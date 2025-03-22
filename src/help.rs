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
    fmt: Formatter,
}

impl<'a> HelpBuilder<'a> {
    pub fn new(args: &'a Args, is_terminal: bool) -> Self {
        Self {
            args,
            specs: args.log().to_vec(), // TODO: filter
            fmt: Formatter::new(is_terminal),
        }
    }

    pub fn build(mut self) -> String {
        self.build_description();
        self.build_usage();
        self.build_example();
        // TODO:  arguments, options, commands
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

        // TODO: and argments, [COMMAND]

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
            format!("-{short}, --{}", spec.name)
        } else {
            format!("          --{}", spec.name)
        };
        self.fmt
            .write(&format!("  {} <{}>\n", self.fmt.bold(&names), spec.ty));
        for line in spec.doc.lines() {
            self.fmt.write(&format!("          {line}\n"));
        }
        if let Some(env) = spec.env {
            self.fmt.write(&format!("          [env: {env}]\n"));
        }
        if let Some(default) = spec.default {
            self.fmt.write(&format!("          [default: {default}]\n"));
        }
    }

    fn build_flag(&mut self, spec: FlagSpec) {
        let names = if let Some(short) = spec.short {
            format!("-{}, --{}", short, spec.name)
        } else {
            format!("    --{}", spec.name)
        };
        self.fmt.write(&format!("  {}\n", self.fmt.bold(&names)));
        for line in spec.doc.lines() {
            self.fmt.write(&format!("          {line}\n"));
        }
        if let Some(env) = spec.env {
            self.fmt.write(&format!("          [env: {env}]\n"));
        }
    }

    fn has_positional_args(&self) -> bool {
        self.specs.iter().any(|spec| matches!(spec, Spec::Opt(_)))
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
  -h, --help
          Print help
      --version
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
  -h, --help
          Print help
  -f, --foo <INTEGER>
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
  -h, --help
          Print help
  -f, --foo <INTEGER>
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
            name: "INT-0",
            doc: "An integer",
            example: Some("3"),
            ..Default::default()
        }
        .take(&mut args);
        ArgSpec {
            name: "INT-1",
            doc: "An integer",
            default: Some("1"),
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
  -h, --help
          Print help
  -f, --foo <INTEGER>
          An integer
"#
        );
    }

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }
}
