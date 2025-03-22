use std::{borrow::Cow, io::IsTerminal};

use crate::{args::Args, log::Spec};

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
        // TODO: example, arguments, options
        self.fmt.text
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
            if self.has_options() { " [OPTIONS]" } else { "" }
        ));

        // TODO: required options, and argments, [COMMAND]

        self.fmt.write("\n\n");
    }

    fn has_options(&self) -> bool {
        self.specs.iter().all(|spec| match spec {
            Spec::Opt(spec) => spec.example.is_none(),
            Spec::Flag(_) => true,
            Spec::Arg(_) | Spec::Subcommand(_) => false,
        })
    }
}

#[derive(Debug, Default)]
struct Formatter {
    text: String,
    is_terminal: bool,
}

impl Formatter {
    fn new(is_terminal: bool) -> Self {
        Self {
            text: String::new(),
            is_terminal,
        }
    }

    fn write(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn bold<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("{BOLD}{}{RESET}", s))
        } else {
            Cow::Borrowed(s)
        }
    }

    fn bold_underline<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("{BOLD}{UNDERLINE}{}{RESET}", s))
        } else {
            Cow::Borrowed(s)
        }
    }
}

// TODO:
const BOLD: &str = "\x1B[1m";
const UNDERLINE: &str = "\x1B[4m";
const RESET: &str = "\x1B[0m";
