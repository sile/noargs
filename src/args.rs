use crate::AppMetadata;

#[derive(Debug)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    show_help: bool,
    next_arg_index: Option<usize>,
}

impl CliArgs {
    pub fn new<I>(raw_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let raw_args = raw_args.skip(1).map(Some).collect::<Vec<_>>();
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
