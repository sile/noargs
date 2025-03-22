use crate::args::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgSpec {
    pub name: &'static str,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
    pub doc: &'static str,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
}

impl ArgSpec {
    pub fn take(self, args: &mut Args) -> Arg {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Arg {
    Positional {
        spec: ArgSpec,
        index: usize,
        value: String,
    },
    Default {
        spec: ArgSpec,
    },
    Example {
        spec: ArgSpec,
    },
    None {
        spec: ArgSpec,
    },
}
