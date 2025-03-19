#[derive(Debug)]
#[expect(dead_code)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    positional_args_start: usize,
    named_args_end: usize,
}

impl CliArgs {
    #[expect(unused_variables)]
    pub fn take_flag(&mut self, flag: CliFlag) -> CliFlag {
        todo!()
    }

    #[expect(unused_variables)]
    pub fn take_option(&mut self, option: CliOption) -> Result<CliOption, TakeOptionError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum TakeOptionError {
    MissingValue { option: CliOption },
    MissingRequiredOption { option: CliOption },
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliOption {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    default_value: String,
    value: Option<String>,
}

impl CliOption {
    //
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliRequiredOption {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    example_value: Option<&'static str>,
    value: String,
}

impl CliRequiredOption {
    //
}

#[derive(Debug)]
#[expect(dead_code)]
pub struct CliFlag {
    long_name: Option<&'static str>,
    short_name: Option<char>,
    doc: Option<&'static str>,
    is_present: bool,
}

impl CliFlag {
    pub const HELP: Self = Self::new("help", 'h').doc("Print help");
    pub const VERSION: Self = Self::long("version").doc("Print version");
    pub const OPTIONS_END: Self = Self::long("").doc("Indicate options end");

    pub const fn new(long_name: &'static str, short_name: char) -> Self {
        Self {
            long_name: Some(long_name),
            short_name: Some(short_name),
            doc: None,
            is_present: false,
        }
    }

    pub const fn long(name: &'static str) -> Self {
        Self {
            long_name: Some(name),
            short_name: None,
            doc: None,
            is_present: false,
        }
    }

    pub const fn short(name: char) -> Self {
        Self {
            long_name: None,
            short_name: Some(name),
            doc: None,
            is_present: false,
        }
    }

    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = Some(doc);
        self
    }

    pub const fn is_present(&self) -> bool {
        self.is_present
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
