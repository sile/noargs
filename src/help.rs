use std::io::IsTerminal;

use crate::{args::Args, flag::FlagSpec, formatter::Formatter, log::Spec, opt::OptSpec};

#[derive(Debug)]
pub struct HelpBuilder<'a> {
    args: &'a Args,
    specs: Vec<Spec>,
    fmt: Formatter,
}

impl<'a> HelpBuilder<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self {
            args,
            specs: args.log().entries.clone(), // TODO: filter
            fmt: Formatter::new(std::io::stdout().is_terminal()),
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
        self.fmt.write("\n");
    }

    fn build_opt(&mut self, spec: OptSpec) {
        // TODO
        self.fmt.write("\n");
    }

    fn build_flag(&mut self, spec: FlagSpec) {
        // TODO
    }

    fn has_options(&self, include_requried: bool) -> bool {
        self.specs.iter().all(|spec| match spec {
            Spec::Opt(spec) => include_requried || spec.example.is_none(),
            Spec::Flag(_) => true,
            Spec::Arg(_) | Spec::Subcommand(_) => false,
        })
    }
}
