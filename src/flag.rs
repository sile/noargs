use std::env::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlagSpec {
    pub long: &'static str,
    pub short: char,
    pub doc: &'static str,
    pub env: &'static str,
    pub before: Option<usize>,
    pub after: Option<usize>,
}

impl FlagSpec {
    pub const DEFAULT: Self = Self {
        long: "",
        short: '\0',
        doc: "",
        env: "",
        before: None,
        after: None,
    };

    pub const HELP: Self = Self {
        long: "help",
        short: 'h',
        doc: "Print help",
        ..Self::DEFAULT
    };

    pub const VERSION: Self = Self {
        long: "version",
        doc: "Print version",
        ..Self::DEFAULT
    };

    pub fn take(self, args: &mut Args) -> Flag {
        todo!()
    }
}

impl Default for FlagSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone)]
pub struct Flag {
    spec: FlagSpec,
    kind: Option<FlagKind>,
    index: Option<usize>,
}

impl Flag {
    pub fn spec(&self) -> FlagSpec {
        self.spec
    }

    pub fn kind(&self) -> Option<FlagKind> {
        self.kind
    }

    pub fn is_present(&self) -> bool {
        self.index.is_some()
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagKind {
    ShortName,
    LongName,
    EnvVar,
}
