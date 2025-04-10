use std::borrow::Cow;

const BOLD: &str = "\x1B[1m";
const UNDERLINE: &str = "\x1B[4m";
const RESET: &str = "\x1B[0m";

#[derive(Debug)]
pub struct Formatter {
    text: String,
    is_terminal: bool,
}

impl Formatter {
    pub fn new(is_terminal: bool) -> Self {
        Self {
            text: String::new(),
            is_terminal,
        }
    }

    pub fn write(&mut self, s: &str) {
        self.text.push_str(s);
    }

    pub fn bold<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("{BOLD}{}{RESET}", s))
        } else {
            Cow::Borrowed(s)
        }
    }

    pub fn bold_underline<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.is_terminal {
            Cow::Owned(format!("{BOLD}{UNDERLINE}{}{RESET}", s))
        } else {
            Cow::Borrowed(s)
        }
    }

    pub fn finish(self) -> String {
        self.text
    }
}
