use std::str::FromStr;

#[derive(Debug)]
pub struct HelpBuilder {}

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
    #[expect(dead_code)]
    help_builder: Option<HelpBuilder>,
}

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
        for arg in &mut raw_args {
            if arg.text == "--" {
                arg.consumed = true;
                arg.positional = true;
                break;
            }
        }
        Self {
            raw_args,
            help_builder: None,
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

    // TODO: option(), subcommand()

    fn try_finish(&self) -> bool {
        self.raw_args.iter().all(|a| a.consumed)
    }

    pub fn finish(self) {
        if self.try_finish() {
            return;
        }

        println!(
            "[error] there are unconsumed argument: {}",
            self.raw_args
                .iter()
                .filter(|a| !a.consumed)
                .map(|a| a.text.clone())
                .collect::<Vec<_>>()
                .join(" ")
        );
        std::process::exit(1);
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
    fn flags() {
        let mut args = Args::with_raw_args(["-b", "--foo", "10"]);

        assert!(args.flag("foo").is_present());
        assert!(!args.flag("bar").is_present());
        assert!(args.flag("baz").short('b').is_present());

        let v = args.arg("INTEGER-0").parse::<usize>();
        assert_eq!(v, 10);

        args.finish();
    }
}
