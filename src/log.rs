use crate::{arg::ArgSpec, flag::FlagSpec, opt::OptSpec, subcommand::SubcommandSpec};

// TODO: remove
#[derive(Debug, Default, Clone)]
pub struct Log {
    pub entries: Vec<Spec>,
}

impl Log {
    pub fn entries(&self) -> &[Spec] {
        &self.entries
    }

    pub fn record_arg(&mut self, spec: ArgSpec) {
        self.entries.push(Spec::Arg(spec));
    }

    pub fn record_opt(&mut self, spec: OptSpec) {
        self.entries.push(Spec::Opt(spec));
    }

    pub fn record_flag(&mut self, spec: FlagSpec) {
        self.entries.push(Spec::Flag(spec));
    }

    pub fn record_subcommand(&mut self, spec: SubcommandSpec) {
        self.entries.push(Spec::Subcommand(spec));
    }
}

// TODO: rename?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spec {
    Arg(ArgSpec),
    Opt(OptSpec),
    Flag(FlagSpec),
    Subcommand(SubcommandSpec),
}
