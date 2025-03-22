use crate::{arg::ArgSpec, flag::FlagSpec, opt::OptSpec, subcommand::SubcommandSpec};

#[derive(Debug, Default, Clone)]
pub struct Log {
    entries: Vec<LogEntry>,
}

impl Log {
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn record_arg(&mut self, spec: ArgSpec) {
        self.entries.push(LogEntry::Arg(spec));
    }

    pub fn record_opt(&mut self, spec: OptSpec) {
        self.entries.push(LogEntry::Opt(spec));
    }

    pub fn record_flag(&mut self, spec: FlagSpec) {
        self.entries.push(LogEntry::Flag(spec));
    }

    pub fn record_subcommand(&mut self, spec: SubcommandSpec) {
        self.entries.push(LogEntry::Subcommand(spec));
    }
}

// TODO: rename?
#[derive(Debug, Clone, Copy)]
pub enum LogEntry {
    Arg(ArgSpec),
    Opt(OptSpec),
    Flag(FlagSpec),
    Subcommand(SubcommandSpec),
}
