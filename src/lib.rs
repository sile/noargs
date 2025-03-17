use std::{
    io::{IsTerminal, Write},
    str::FromStr,
};

mod args;
mod metadata;

pub use self::args::CliArgs;
pub use self::metadata::AppMetadata;

#[derive(Debug)]
pub struct HelpBuilder {
    pkg_name: String,
    description: Option<String>,
    has_options: bool,

    // TODO: handle optional and multi
    arg_names: Vec<String>,
}

impl HelpBuilder {
    pub fn output<W>(&self, mut writer: W, _is_termina: bool) -> std::io::Result<()>
    where
        W: Write,
    {
        if let Some(s) = &self.description {
            writeln!(writer, "{s}\n")?;
        }

        write!(
            writer,
            "{} {}",
            bold_underline("Usage:"),
            bold(&self.pkg_name)
        )?;
        if self.has_options {
            write!(writer, " [OPTIONS]")?;
        }
        for arg_name in &self.arg_names {
            write!(writer, " [{arg_name}]")?;
        }
        writeln!(writer)?;

        if !self.arg_names.is_empty() {
            writeln!(writer, "{}:", bold_underline("Arguments:"))?;
            // TODO
        }

        if self.has_options {
            writeln!(writer, "{}:", bold_underline("Options:"))?;
        }

        // TODO: subcommand

        Ok(())
    }
}

impl Default for HelpBuilder {
    fn default() -> Self {
        Self {
            pkg_name: env!("CARGO_PKG_NAME").to_owned(),
            // TODO: pkg_version: env!("CARGO_PKG_VERSION").to_owned(),
            description: None,
            has_options: false,
            arg_names: Vec::new(),
        }
    }
}

// TODO
fn bold_underline(s: &str) -> &str {
    s
}

fn bold(s: &str) -> &str {
    s
}

#[derive(Debug)]
pub struct RawArg {
    pub text: String,
    pub consumed: bool,
    pub positional: bool,
}

#[derive(Debug)]
pub struct Args {
    raw_args: Vec<RawArg>,
    // TDOO: version, app_name
    help_builder: HelpBuilder,
    show_help: bool,
}

// TODO: impl Drop (for finish())

impl Args {
    pub fn new() -> Self {
        Self::with_raw_args(std::env::args().skip(1))
    }

    pub fn with_raw_args<I, T>(raw_args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut raw_args = raw_args
            .into_iter()
            .map(|a| RawArg {
                text: a.into(),
                consumed: false,
                positional: false,
            })
            .collect::<Vec<_>>();
        let mut show_help = false;
        for arg in &mut raw_args {
            if arg.text == "--" {
                arg.consumed = true;
                arg.positional = true;
                break;
            }
            if matches!(arg.text.as_str(), "-h" | "--help") {
                show_help = true;
            }
        }
        Self {
            raw_args,
            help_builder: HelpBuilder::default(),
            show_help,
        }
    }

    // fn example() for help

    pub fn with_version(self) -> Self {
        todo!()
    }

    fn next_positional_raw_arg(&mut self) -> Option<&mut RawArg> {
        let offset = self.raw_args.iter().position(|a| a.positional).unwrap_or(0);
        self.raw_args.iter_mut().skip(offset).find(|a| !a.consumed)
    }

    pub fn arg(&mut self, name: &str) -> PositionalArg {
        PositionalArg::new(self, name)
    }

    pub fn flag(&mut self, name: &str) -> Flag {
        Flag::new(self, name)
    }

    pub fn option(&mut self, name: &str) -> OptionArg {
        OptionArg::new(self, name)
    }

    pub fn subcommand<'a>(&mut self, name: &str) -> Subcommand {
        Subcommand::new(self, name)
    }

    fn try_finish(&self) -> bool {
        self.raw_args.iter().all(|a| a.consumed)
    }

    pub fn help(&self) -> String {
        let mut buf = Vec::new();
        self.help_builder
            .output(&mut buf, false)
            .expect("infallible");
        String::from_utf8(buf).expect("infallible")
    }

    // TODO: rename
    fn finish_inner(&mut self) {
        if self.show_help {
            let stdout = std::io::stdout();
            let is_terminal = stdout.is_terminal();
            self.help_builder
                .output(stdout.lock(), is_terminal)
                .expect("TODO");
            std::process::exit(0);
        }

        if self.try_finish() {
            return;
        }

        println!(
            "[error] there are unknown or unconsumed arguments: {}",
            self.raw_args
                .iter()
                .filter(|a| !a.consumed)
                .map(|a| a.text.clone())
                .collect::<Vec<_>>()
                .join(" ")
        );
        std::process::exit(1);
    }

    pub fn finish(mut self) {
        self.finish_inner();
    }
}

impl Drop for Args {
    fn drop(&mut self) {
        self.finish_inner();
    }
}

#[derive(Debug)]
pub struct OptionalPositionalArg<'a> {
    inner: PositionalArg<'a>,
}

impl<'a> OptionalPositionalArg<'a> {
    pub fn parse<T>(mut self) -> Option<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.inner.try_parse() {
            Err(e) => {
                eprintln!(
                    "[error] invalid argument {}: value={}, reason={}",
                    self.inner.name, e.arg, e.error
                );
                std::process::exit(1);
            }
            Ok(value) => value,
        }
    }

    // TODO: fn parse_or_default()
}

#[derive(Debug)]
pub struct Flag<'a> {
    args: &'a mut Args,
    name: String,
    short_name: Option<char>,
}

impl<'a> Flag<'a> {
    fn new(args: &'a mut Args, name: &str) -> Self {
        Self {
            args,
            name: name.to_owned(),
            short_name: None,
        }
    }

    pub fn short(mut self, name: char) -> Self {
        self.short_name = Some(name);
        self
    }

    pub fn is_present(self) -> bool {
        // TODO: duplicate check
        let mut present = false;
        for arg in &mut self.args.raw_args {
            if arg.positional {
                break;
            }
            if arg.consumed {
                continue;
            };

            // TODO: optimize
            if arg.text == format!("--{}", self.name) {
                present = true;
                arg.consumed = true;
            } else if let Some(short) = self.short_name {
                if arg.text == format!("-{}", short) {
                    present = true;
                    arg.consumed = true;
                }
            }
        }
        present
    }
}

// TODO: s/PositionalArg/CliArg/
#[derive(Debug)]
pub struct PositionalArg<'a> {
    args: &'a mut Args,
    name: String,
}

impl<'a> PositionalArg<'a> {
    fn new(args: &'a mut Args, name: &str) -> Self {
        Self {
            args,
            name: name.to_owned(),
        }
    }

    pub fn optional(self) -> OptionalPositionalArg<'a> {
        OptionalPositionalArg { inner: self }
    }

    pub fn help(self, _help: &str) -> Self {
        // TODO
        self
    }

    // TODO: multi().at_least(1).at_most(10)

    fn try_parse<T>(&mut self) -> Result<Option<T>, ParseError<T::Err>>
    where
        T: FromStr,
    {
        let Some(arg) = self.args.next_positional_raw_arg() else {
            return Ok(None);
        };
        arg.positional = true;
        arg.consumed = true;
        arg.text.parse().map(Some).map_err(|error| ParseError {
            arg: arg.text.clone(),
            error,
        })
    }

    pub fn parse<T>(mut self) -> T
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.try_parse() {
            Err(e) => {
                eprintln!(
                    "[error] invalid argument {}: value={}, reason={}",
                    self.name, e.arg, e.error
                );
                std::process::exit(1);
            }
            Ok(None) => {
                eprintln!("[error] missing positional argument {}", self.name);
                std::process::exit(1);
            }
            Ok(Some(value)) => value,
        }
    }
}

#[derive(Debug)]
pub struct Subcommand<'a> {
    args: &'a mut Args,
    name: String,
    // TODO: help
}

impl<'a> Subcommand<'a> {
    fn new(args: &'a mut Args, name: &str) -> Self {
        Self {
            args,
            name: name.to_owned(),
        }
    }

    pub fn is_present(self) -> bool {
        for (i, arg) in self.args.raw_args.iter().enumerate() {
            if arg.consumed {
                continue;
            }
            if arg.positional {
                panic!(); // TODO: error handling
            }

            // TODO: error handling
            assert!(self.args.raw_args.iter().skip(i + 1).all(|a| !a.consumed));

            if arg.text != self.name {
                return false;
            }

            self.args.raw_args[i].consumed = true;
            break;
        }

        true
    }
}

// TODO: s/OptionArg/CliOption/
#[derive(Debug)]
pub struct OptionArg<'a> {
    args: &'a mut Args,
    name: String,
    short_name: Option<char>,
    // TODO: value_name
}

impl<'a> OptionArg<'a> {
    fn new(args: &'a mut Args, name: &str) -> Self {
        Self {
            args,
            name: name.to_owned(),
            short_name: None,
        }
    }

    pub fn short(mut self, name: char) -> Self {
        self.short_name = Some(name);
        self
    }

    pub fn help(self, _help: &str) -> Self {
        // TODO
        self
    }

    // TODO: required()
    // TODO: multi().at_least(1).at_most(10)

    fn try_parse<T>(&mut self) -> Result<Option<T>, ParseError<T::Err>>
    where
        T: FromStr,
    {
        let Some(i) = self.args.raw_args.iter().position(|arg| {
            if arg.positional || arg.consumed {
                return false;
            }

            // TODO: optimize
            // TODO: `--key=value` syntax
            arg.text == format!("--{}", self.name)
                || self
                    .short_name
                    .as_ref()
                    .is_some_and(|name| arg.text == format!("-{name}"))
        }) else {
            return Ok(None);
        };

        let key = &mut self.args.raw_args[i];
        key.consumed = true;

        let Some(value) = self.args.raw_args.get_mut(i + 1) else {
            // return Err(ParseError {
            //     arg: key.text.clone(),
            //     error: todo!(),
            // });
            todo!("no value error")
        };

        value.consumed = true;
        value.text.parse().map(Some).map_err(|error| ParseError {
            arg: value.text.clone(),
            error,
        })
    }

    pub fn parse<T>(mut self) -> Option<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.try_parse() {
            Err(e) => {
                // TODO:
                eprintln!(
                    "[error] invalid argument --{}: value={}, reason={}",
                    self.name, e.arg, e.error
                );
                std::process::exit(1);
            }
            Ok(value) => value,
        }
    }
}

#[derive(Debug)]
pub struct ParseError<E> {
    pub arg: String,
    pub error: E,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional_args() {
        let mut args = Args::with_raw_args(["10", "3"]);

        let v = args.arg("INTEGER-0").parse::<usize>();
        assert_eq!(v, 10);

        let v = args.arg("INTEGER-1").parse::<usize>();
        assert_eq!(v, 3);

        assert!(matches!(
            args.arg("INTEGER-2").try_parse::<usize>(),
            Ok(None)
        ));

        args.finish();
    }

    #[test]
    fn optional_positional_args() {
        let mut args = Args::with_raw_args(["10"]);

        let v = args.arg("INTEGER-0").optional().parse::<usize>();
        assert_eq!(v, Some(10));

        let v = args.arg("INTEGER-1").optional().parse::<usize>();
        assert_eq!(v, None);

        args.finish();
    }

    #[test]
    fn option_args() {
        let mut args = Args::with_raw_args(["--foo", "10", "-b", "1", "3"]);

        let v = args.option("foo").parse::<usize>();
        assert_eq!(v, Some(10));

        let v = args.option("bar").short('b').parse::<usize>();
        assert_eq!(v, Some(1));

        let v = args.arg("INTEGER-1").parse::<usize>();
        assert_eq!(v, 3);

        args.finish();
    }

    #[test]
    fn flags() {
        let mut args = Args::with_raw_args(["-b", "--foo", "10"]);

        assert!(args.flag("foo").is_present());
        assert!(!args.flag("bar").is_present());
        assert!(args.flag("baz").short('b').is_present());

        let v = args.arg("INTEGER-0").parse::<usize>();
        assert_eq!(v, 10);

        args.finish();
    }

    #[test]
    fn subcommand() {
        let mut args = Args::with_raw_args(["--foo", "run", "--bar", "10"]);

        assert!(args.flag("foo").is_present());
        if args.subcommand("show").is_present() {
        } else if args.subcommand("run").is_present() {
            let v = args.option("bar").short('b').parse::<usize>();
            assert_eq!(v, Some(10));
        }
    }
}
