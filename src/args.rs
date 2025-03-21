use crate::error::Error;

#[derive(Debug)]
pub struct Args {
    metadata: Metadata,
    raw_args: Vec<RawArg>,
}

impl Args {
    pub fn new<I>(args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let raw_args = args
            .enumerate()
            .map(|(i, value)| RawArg {
                value: (i != 0).then_some(value),
            })
            .collect();
        Self {
            metadata: Metadata::default(),
            raw_args,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn raw_args(&self) -> &[RawArg] {
        &self.raw_args
    }

    pub fn finish(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RawArg {
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub app_name: &'static str,
    pub app_description: &'static str,
    pub help_option_name: Option<&'static str>, // TODO: OptSpec
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            app_name: env!("CARGO_PKG_NAME"),
            app_description: env!("CARGO_PKG_DESCRIPTION"),
            help_option_name: None,
        }
    }
}
