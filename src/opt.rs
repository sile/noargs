use crate::args::{Args, Metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptSpec {
    pub name: &'static str, // TODO: Option?
    pub short: Option<char>,
    pub ty: &'static str,
    pub doc: &'static str,
    pub env: Option<&'static str>,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
}

impl OptSpec {
    pub const DEFAULT: Self = Self {
        name: "",
        short: None,
        ty: "VALUE",
        doc: "",
        env: None,
        default: None,
        example: None,
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Opt {
        self.metadata = args.metadata();
        args.log_mut().record_opt(self);
        todo!()
    }
}

impl Default for OptSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opt {
    Long {
        spec: OptSpec,
        index: usize,
        value: String,
    },
    Short {
        spec: OptSpec,
        index: usize,
        value: String,
    },
    Env {
        spec: OptSpec,
    },
    Default {
        spec: OptSpec,
    },
    Example {
        spec: OptSpec,
    },
    None,
}
