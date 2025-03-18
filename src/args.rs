use std::{borrow::Cow, io::IsTerminal};

#[derive(Debug, Clone)]
struct OptionSpec {
    long_name: String,
    short_name: Option<char>,
    doc: Option<String>, // TODO: rename
}

impl OptionSpec {
    fn new(long_name: &str) -> Self {
        Self {
            long_name: long_name.to_owned(),
            short_name: None,
            doc: None,
        }
    }
}

#[derive(Debug)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    named_args_end: usize,
    metadata: Metadata,
    options: Vec<OptionSpec>,
}

impl CliArgs {
    pub fn new<I>(raw_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut raw_args = raw_args.skip(1).map(Some).collect::<Vec<_>>();
        let mut named_args_end = raw_args.len();
        for (i, raw_arg) in raw_args.iter_mut().enumerate() {
            if raw_arg.as_ref().is_some_and(|a| a == "--") {
                *raw_arg = None;
                named_args_end = i;
                break;
            }
        }
        Self {
            raw_args,
            named_args_end,
            metadata: Metadata::default(),
            options: Vec::new(),
        }
    }

    pub fn from_slice(raw_args: &[&str]) -> Self {
        Self::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn take_flag(&mut self, spec: OptionSpec) -> bool {
        self.options.push(spec.clone()); // TODO: remove clone

        for raw_arg in &mut self.raw_args[..self.named_args_end] {
            let found = raw_arg.take_if(|raw_arg| {
                if raw_arg.starts_with("--") && &raw_arg[2..] == spec.long_name {
                    true
                } else if spec.short_name.is_some()
                    && raw_arg.starts_with('-')
                    && raw_arg.chars().count() == 2
                    && raw_arg.chars().nth(1) == spec.short_name
                {
                    true
                } else {
                    false
                }
            });
            if found.is_some() {
                return true;
            }
        }
        false
    }

    pub fn version(&mut self) -> Version {
        Version::new(self)
    }

    pub fn help(&mut self) -> Help {
        Help::new(self)
    }

    pub fn output(self) -> Output {
        // TODO: check unknown (unconsumed) args
        Output::new(self)
    }

    pub fn metadata(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

// TODO: move
// TODO: Rename to App (?)
#[derive(Debug)]
pub struct Metadata {
    pub app_name: &'static str,
    pub app_version: &'static str,
    pub app_description: &'static str,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            app_name: env!("CARGO_PKG_NAME"),
            app_version: env!("CARGO_PKG_VERSION"),
            app_description: env!("CARGO_PKG_DESCRIPTION"),
        }
    }
}

// TODO: move
#[derive(Debug)]
pub struct Version<'a> {
    args: &'a mut CliArgs,
}

impl<'a> Version<'a> {
    fn new(args: &'a mut CliArgs) -> Self {
        Self { args }
    }

    pub fn is_present(self) -> bool {
        let spec = OptionSpec {
            doc: Some("Print version".to_owned()),
            ..OptionSpec::new("version")
        };
        self.args.take_flag(spec)
    }
}

#[derive(Debug)]
pub struct Help<'a> {
    args: &'a mut CliArgs,
    short_name: Option<char>,
}

impl<'a> Help<'a> {
    fn new(args: &'a mut CliArgs) -> Self {
        Self {
            args,
            short_name: None,
        }
    }

    pub fn short(mut self, name: char) -> Self {
        self.short_name = Some(name);
        self
    }

    pub fn is_present(self) -> bool {
        let spec = OptionSpec {
            short_name: self.short_name,
            doc: Some("Print help".to_owned()),
            ..OptionSpec::new("help")
        };
        self.args.take_flag(spec)
    }
}

// TODO: move
pub struct Output {
    args: CliArgs,
    is_terminal: bool,
}

impl Output {
    fn new(args: CliArgs) -> Self {
        Self {
            args,
            is_terminal: false,
        }
    }

    pub fn for_stdout(mut self) -> Self {
        self.is_terminal = std::io::stdout().is_terminal();
        self
    }

    pub fn for_stderr(mut self) -> Self {
        self.is_terminal = std::io::stderr().is_terminal();
        self
    }

    pub fn version_line(&self) -> String {
        format!(
            "{} {}",
            self.args.metadata.app_name, self.args.metadata.app_version
        )
    }

    pub fn help_text(&self) -> String {
        let mut text = format!("{}\n\n", self.args.metadata.app_description);

        text.push_str(&format!(
            "{} {} [OPTIONS]",
            self.bold_underline("Usage:"),
            self.bold(self.args.metadata.app_name)
        ));
        // TODO: ARGUMENTS
        text.push_str("\n\n");

        text.push_str(&self.bold_underline("Options:"));
        for option in &self.args.options {
            text.push_str("\n  ");
            if let Some(name) = option.short_name {
                text.push_str(&self.bold(&format!("-{}, ", name)));
            } else {
                text.push_str("    ");
            }
            text.push_str(&self.bold(&format!("--{}", option.long_name)));
            // TODO: value_name
            if let Some(doc) = &option.doc {
                for line in doc.lines() {
                    text.push_str(&format!("\n          {}", line));
                }
            }
        }
        text.push_str("\n");

        text
    }

    fn bold_underline<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if !self.is_terminal {
            Cow::Borrowed(s)
        } else {
            todo!()
        }
    }

    fn bold<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if !self.is_terminal {
            Cow::Borrowed(s)
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let mut args = CliArgs::from_slice(&["test", "run"]);
        assert!(!args.version().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--", "--version"]);
        assert!(!args.version().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--version"]);
        assert!(args.version().is_present());
        assert!(!args.version().is_present());

        args.metadata().app_name = "test";
        args.metadata().app_version = "0.0.1";
        assert_eq!(args.output().version_line(), "test 0.0.1");
    }

    #[test]
    fn help() {
        let mut args = CliArgs::from_slice(&["test", "run"]);
        assert!(!args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--help"]);
        assert!(args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "-h"]);
        assert!(!args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "-h"]);
        args.metadata().app_description = "This is a test";
        args.metadata().app_name = "test";
        args.version().is_present();

        assert!(args.help().short('h').is_present());
        assert_eq!(
            args.output().help_text(),
            r#"This is a test

Usage: test [OPTIONS]

Options:
      --version
          Print version
  -h, --help
          Print help
"#
        );
    }
}
