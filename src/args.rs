use std::io::Write;

use crate::{writer::DefaultWriter, AppMetadata};

#[derive(Debug)]
pub struct CliArgs<W = DefaultWriter> {
    writer: W,
    raw_args: Vec<Option<String>>,
    show_help: bool,
    next_arg_index: Option<usize>,
}

impl CliArgs<DefaultWriter> {
    pub fn new() -> Self {
        Self::with_writer_and_raw_args(DefaultWriter::new(), std::env::args())
    }
}

impl<W: Write> CliArgs<W> {
    fn with_writer_and_raw_args<I, T>(writer: W, raw_args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let raw_args = raw_args
            .into_iter()
            .skip(1)
            .map(|a| Some(a.into()))
            .collect::<Vec<_>>();
        Self {
            writer,
            raw_args,
            show_help: false,
            next_arg_index: None,
        }
    }

    pub fn metadata(&mut self) -> AppMetadata<W> {
        AppMetadata::new(self)
    }

    // TODO: help
}
