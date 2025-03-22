use crate::args::{Args, Metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubcommandSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub min_index: Option<usize>,
    pub max_index: Option<usize>,
    pub metadata: Metadata,
}

impl SubcommandSpec {
    pub const DEFAULT: Self = Self {
        name: "",
        doc: "",
        min_index: None,
        max_index: None,
        metadata: Metadata::DEFAULT,
    };

    pub fn take(mut self, args: &mut Args) -> Subcommand {
        self.metadata = args.metadata();
        args.log_mut().record_subcommand(self);

        for (index, raw_arg) in args.range_mut(self.min_index, self.max_index) {
            let Some(value) = &raw_arg.value else {
                continue;
            };

            if value == self.name {
                raw_arg.value = None;
                return Subcommand::Some { spec: self, index };
            }
        }

        Subcommand::None { spec: self }
    }
}

impl Default for SubcommandSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subcommand {
    Some { spec: SubcommandSpec, index: usize },
    None { spec: SubcommandSpec },
}

impl Subcommand {
    pub fn spec(self) -> SubcommandSpec {
        match self {
            Subcommand::Some { spec, .. } | Subcommand::None { spec } => spec,
        }
    }

    pub fn ok(self) -> Option<Self> {
        self.is_present().then_some(self)
    }

    pub fn index(self) -> Option<usize> {
        if let Self::Some { index, .. } = self {
            Some(index)
        } else {
            None
        }
    }

    pub fn is_present(self) -> bool {
        matches!(self, Self::Some { .. })
    }
}
