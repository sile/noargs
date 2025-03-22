use crate::flag::FlagSpec;

#[derive(Debug, Default, Clone)]
pub struct Log {
    entries: Vec<LogEntry>,
}

impl Log {
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn record_flag(&mut self, spec: FlagSpec, index: Option<usize>) {
        self.entries.push(LogEntry::Flag { spec, index });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogEntry {
    Flag {
        spec: FlagSpec,
        index: Option<usize>,
    },
}
