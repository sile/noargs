use std::{collections::VecDeque, str::FromStr};

#[derive(Debug)]
pub struct HelpBuilder {}

#[derive(Debug)]
pub struct Args {
    raw_args: VecDeque<String>,
    // TDOO: version, app_name
    need_positional_args: bool,

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
        Self {
            raw_args: raw_args.into_iter().map(|a| a.into()).collect(),
            need_positional_args: false,
            help_builder: None,
        }
    }

    // fn example() for help

    pub fn with_version(self) -> Self {
        todo!()
    }

    pub fn arg(&mut self, name: &str) -> PositionalArg {
        PositionalArg::new(self, name)
    }

    fn try_finish(&self) -> bool {
        self.raw_args.is_empty()
    }

    pub fn finish(self) {
        if self.try_finish() {
            return;
        }

        println!(
            "[error] there are unconsumed argument: {}",
            self.raw_args.into_iter().collect::<Vec<_>>().join(" ")
        );
        std::process::exit(1);
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

    fn try_parse<T>(&mut self) -> Result<Option<T>, ParseError<T::Err>>
    where
        T: FromStr,
    {
        // TODO: handle "--"

        self.args.need_positional_args = true;
        let Some(arg) = self.args.raw_args.pop_front() else {
            return Ok(None);
        };
        arg.parse()
            .map(Some)
            .map_err(|error| ParseError { arg, error })
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

        args.finish();
    }
}
