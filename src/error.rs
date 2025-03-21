use std::{borrow::Cow, io::IsTerminal};

use crate::args::Args;

pub enum Error {
    UnexpectedArg(Args),
    // UnexpectedSubcommand
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = Formatter::new(std::io::stderr().is_terminal());
        match self {
            Error::UnexpectedArg(args) => fmt.format_unexpected_arg(args),
        }
        write!(f, "{}", fmt.text)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = Formatter::new(false);
        match self {
            Error::UnexpectedArg(args) => fmt.format_unexpected_arg(args),
        }
        write!(f, "{}", fmt.text)
    }
}

impl std::error::Error for Error {}

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

    fn format_unexpected_arg(&mut self, args: &Args) {
        self.write(&format!(
            "unexpected argument '{}' found",
            self.bold(args.next_raw_arg_value().expect("infallible"))
        ));

        if let Some(help) = args.metadata().help_option_name {
            self.write(&format!(
                "\nTry '{}' for more information.",
                self.bold(&format!("{} --{}", args.metadata().app_name, help))
            ));
        }
    }

    fn write(&mut self, s: &str) {
        self.text.push_str(s);
    }

    fn bold<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("\x1b[1m{}\x1b[0m", s))
        } else {
            Cow::Borrowed(s)
        }
    }
}
