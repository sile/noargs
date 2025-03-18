use std::{borrow::Cow, io::IsTerminal};

pub trait OrExit {
    type Item;
    fn or_exit(self) -> Self::Item;
}

impl<T, E> OrExit for Result<T, E>
where
    E: std::fmt::Display,
{
    type Item = T;

    fn or_exit(self) -> Self::Item {
        match self {
            Ok(item) => item,
            Err(e) => {
                // TODO: is_terminal
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}

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
    positional_args_start: usize,
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
        let mut positional_args_start = 0;
        for (i, raw_arg) in raw_args.iter_mut().enumerate() {
            if raw_arg.as_ref().is_some_and(|a| a == "--") {
                *raw_arg = None;
                named_args_end = i;
                positional_args_start = i;
                break;
            }
        }
        Self {
            raw_args,
            named_args_end,
            positional_args_start,
            metadata: Metadata::default(),
            options: Vec::new(),
        }
    }

    pub fn from_slice(raw_args: &[&str]) -> Self {
        Self::new(raw_args.iter().map(|a| a.to_string()))
    }

    pub fn arg(&mut self, name: &str) -> CliArg {
        CliArg::new(self, name)
    }

    // TODO: don't take
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
        let mut text = String::new();
        if !self.args.metadata.app_description.is_empty() {
            text.push_str(&format!("{}\n\n", self.args.metadata.app_description));
        }

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

#[derive(Debug)]
pub struct CliArgValue {
    name: String,
    value: String,
}

impl CliArgValue {
    pub fn parse<T>(self) -> Result<T, ParseError<T::Err>>
    where
        T: std::str::FromStr,
    {
        self.value.parse().map_err(|error| ParseError {
            name: self.name,
            error,
        })
    }

    pub fn into_string(self) -> String {
        self.value
    }
}

// TODO: rename
#[derive(Debug)]
pub struct CliArg<'a> {
    args: &'a mut CliArgs,
    name: String,
}

impl<'a> CliArg<'a> {
    fn new(args: &'a mut CliArgs, name: &str) -> Self {
        Self {
            args,
            name: name.to_owned(),
        }
    }

    pub fn take(self) -> Result<CliArgValue, TakeError> {
        let raw_arg = self.args.raw_args[self.args.positional_args_start..]
            .iter_mut()
            .inspect(|_| self.args.positional_args_start += 1)
            .find_map(|a| a.take())
            .ok_or(TakeError)?;
        Ok(CliArgValue {
            name: format!("<{}>", self.name),
            value: raw_arg,
        })
    }
}

#[derive(Debug)]
pub struct ParseError<E> {
    name: String,
    error: E,
}

impl<E: std::fmt::Display> std::fmt::Display for ParseError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "invalid argument {:?}: {}", self.name, self.error)
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for ParseError<E> {}

#[derive(Debug)]
pub struct TakeError;

impl std::fmt::Display for TakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "no more arguments")
    }
}

impl std::error::Error for TakeError {}

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

    #[test]
    fn required_positional_args() {
        let mut args = CliArgs::from_slice(&["test", "8"]);
        let v: usize = args.arg("INT").take().or_exit().parse().or_exit();
        assert_eq!(v, 8);
        assert!(args.arg("INT").take().is_err());

        let mut args = CliArgs::from_slice(&["test", "--help"]);
        args.metadata().app_name = "test";
        assert!(args.help().is_present());
        assert!(args.arg("INT").take().is_err());

        assert_eq!(
            args.output().help_text(),
            r#"Usage: test [OPTIONS]

Options:
      --help
          Print help
"#
        );
    }
}
