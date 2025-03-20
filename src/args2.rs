use std::str::FromStr;

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    positional_args_start: usize,
    named_args_end: usize,
}

impl CliArgs {
    pub fn take_arg<T: FromStr>(&mut self, spec: Arg) -> Result<T, ParseError<T::Err>> {
        todo!()
    }

    pub fn take_option_arg<T: FromStr>(
        &mut self,
        spec: OptionArg,
    ) -> Result<Option<T>, ParseError<T::Err>> {
        todo!()
    }

    pub fn take_flag(&mut self, spec: Flag) -> bool {
        todo!()
    }

    pub fn take_subcommand(&mut self, spec: Subcommand) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct ParseError<E> {
    // TODO: arg info
    pub error: E,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Arg {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: Option<&'static str>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
    hidden_env: Option<&'static str>,
    default_value: Option<&'static str>,
    example_value: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct OptionArg {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    value_name: Option<&'static str>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
    hidden_env: Option<&'static str>,
    default_value: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Flag {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    env: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct Subcommand {
    name: &'static str,
    doc: Option<&'static str>,
}
