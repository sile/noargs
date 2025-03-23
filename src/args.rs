use std::borrow::Cow;

use crate::{arg::Arg, cmd::Cmd, error::Error, flag::Flag, opt::Opt};

#[derive(Debug)]
pub struct Args {
    metadata: Metadata,
    raw_args: Vec<RawArg>,
    log: Vec<Taken>,
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
            log: Vec::new(),
        }
    }

    pub fn metadata(&self) -> Metadata {
        self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn remaining_args(&self) -> impl '_ + Iterator<Item = (usize, &str)> {
        self.raw_args
            .iter()
            .enumerate()
            .filter_map(|(i, a)| a.value.as_ref().map(|v| (i, v.as_str())))
    }

    pub fn finish(self) -> Result<Option<String>, Error> {
        todo!()
    }

    pub(crate) fn raw_args_mut(&mut self) -> &mut [RawArg] {
        &mut self.raw_args
    }

    pub(crate) fn range_mut(
        &mut self,
        min_index: Option<usize>,
        max_index: Option<usize>,
    ) -> impl '_ + Iterator<Item = (usize, &mut RawArg)> {
        self.raw_args_mut()
            .iter_mut()
            .enumerate()
            .take(max_index.map(|i| i + 1).unwrap_or(usize::MAX))
            .skip(min_index.unwrap_or(0))
    }

    pub(crate) fn log(&self) -> &[Taken] {
        &self.log
    }

    pub(crate) fn with_record_arg<F>(&mut self, f: F) -> Arg
    where
        F: FnOnce(&mut Self) -> Arg,
    {
        let arg = f(self);
        self.log.push(Taken::Arg(arg.clone()));
        arg
    }

    pub(crate) fn with_record_opt<F>(&mut self, f: F) -> Opt
    where
        F: FnOnce(&mut Self) -> Opt,
    {
        let opt = f(self);
        self.log.push(Taken::Opt(opt.clone()));
        opt
    }

    pub(crate) fn with_record_flag<F>(&mut self, f: F) -> Flag
    where
        F: FnOnce(&mut Self) -> Flag,
    {
        let flag = f(self);
        self.log.push(Taken::Flag(flag));
        flag
    }

    pub(crate) fn with_record_cmd<F>(&mut self, f: F) -> Cmd
    where
        F: FnOnce(&mut Self) -> Cmd,
    {
        let cmd = f(self);
        self.log.push(Taken::Cmd(cmd));
        cmd
    }

    pub(crate) fn next_raw_arg_value(&self) -> Option<&str> {
        self.raw_args
            .iter()
            .find_map(|a| a.value.as_ref().map(|s| s.as_str()))
    }
}

#[derive(Debug, Clone)]
pub struct RawArg {
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub app_name: &'static str,
    pub app_description: &'static str,
    pub help_flag_name: Option<&'static str>,
    pub help_mode: bool,
}

impl Metadata {
    pub const DEFAULT: Self = Self {
        app_name: env!("CARGO_PKG_NAME"),
        app_description: env!("CARGO_PKG_DESCRIPTION"),
        help_flag_name: Some("help"),
        help_mode: false,
    };

    pub fn version_line(self) -> String {
        format!("{} {}", self.app_name, env!("CARGO_PKG_VERSION"))
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Taken {
    Arg(Arg),
    Opt(Opt),
    Flag(Flag),
    Cmd(Cmd),
}

impl Taken {
    pub fn name(&self) -> &'static str {
        match self {
            Taken::Arg(arg) => arg.spec().name,
            Taken::Opt(opt) => opt.spec().name,
            Taken::Flag(flag) => flag.spec().name,
            Taken::Cmd(cmd) => cmd.spec().name,
        }
    }

    pub fn example(&self) -> Option<Cow<'static, str>> {
        match self {
            Taken::Arg(arg) => arg.spec().example.map(Cow::Borrowed),
            Taken::Opt(opt) => opt
                .spec()
                .example
                .map(|v| Cow::Owned(format!("--{} {v}", opt.spec().name))),
            Taken::Cmd(cmd) if cmd.is_present() => Some(Cow::Borrowed(cmd.spec().name)),
            _ => None,
        }
    }

    pub fn contains_index(&self, index: usize) -> bool {
        (self.min_index().unwrap_or(0)..=self.max_index().unwrap_or(usize::MAX)).contains(&index)
    }

    fn min_index(&self) -> Option<usize> {
        match self {
            Taken::Arg(x) => x.spec().min_index,
            Taken::Opt(x) => x.spec().min_index,
            Taken::Flag(x) => x.spec().min_index,
            Taken::Cmd(x) => x.spec().min_index,
        }
    }

    fn max_index(&self) -> Option<usize> {
        match self {
            Taken::Arg(x) => x.spec().max_index,
            Taken::Opt(x) => x.spec().max_index,
            Taken::Flag(x) => x.spec().max_index,
            Taken::Cmd(x) => x.spec().max_index,
        }
    }
}
