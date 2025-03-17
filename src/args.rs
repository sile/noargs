use std::io::Write;

use crate::AppMetadata;

#[derive(Debug)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    show_help: bool,
    next_arg_index: Option<usize>,
}

impl CliArgs {
    pub fn new() -> Self {
        Self::with_raw_args(std::env::args().skip(1))
    }

    fn with_raw_args<I, T>(raw_args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let raw_args = raw_args
            .into_iter()
            .map(|a| Some(a.into()))
            .collect::<Vec<_>>();
        Self {
            raw_args,
            show_help: false,
            next_arg_index: None,
        }
    }

    pub fn metadata(&mut self) -> AppMetadata {
        AppMetadata::new(self)
    }

    // TODO: help
}
