use crate::{args::Args, flag::FlagSpec, formatter::Formatter, log::Spec, opt::OptSpec};

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
            specs: args.log().entries.clone(), // TODO: filter
            fmt: Formatter::new(is_terminal),
        }
    }

    pub fn build(mut self) -> String {
        self.build_description();
        self.build_usage();
        // TODO: example, arguments, options, commands
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
            "{} {}{}",
            self.fmt.bold_underline("Usage:"),
            self.fmt.bold(self.args.metadata().app_name),
            if self.has_options(false) {
                " [OPTIONS]"
            } else {
                ""
            }
        ));

        // TODO: required options, and argments, [COMMAND]

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

    fn has_options(&self, include_requried: bool) -> bool {
        self.specs.iter().all(|spec| match spec {
            Spec::Opt(spec) => include_requried || spec.example.is_none(),
            Spec::Flag(_) => true,
            Spec::Arg(_) | Spec::Subcommand(_) => false,
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

    fn args(raw_args: &[&str]) -> Args {
        Args::new(raw_args.iter().map(|a| a.to_string()))
    }
}
