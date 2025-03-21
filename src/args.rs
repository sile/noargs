#[derive(Debug)]
pub struct Args {
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
        Self { raw_args }
    }

    pub fn raw_args(&self) -> &[RawArg] {
        &self.raw_args
    }
}

#[derive(Debug, Clone)]
pub struct RawArg {
    pub value: Option<String>,
}
