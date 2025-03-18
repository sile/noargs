#[derive(Debug)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    //show_help: bool,
    named_args_end: usize,
    metadata: Metadata,
}

impl CliArgs {
    pub fn new<I>(raw_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut raw_args = raw_args.skip(1).map(Some).collect::<Vec<_>>();
        let mut named_args_end = raw_args.len();
        for (i, raw_arg) in raw_args.iter_mut().enumerate() {
            if raw_arg.as_ref().is_some_and(|a| a == "--") {
                *raw_arg = None;
                named_args_end = i;
                break;
            }
        }
        Self {
            raw_args,
            //show_help: false,
            named_args_end,
            metadata: Metadata::default(),
        }
    }

    pub fn from_slice(raw_args: &[&str]) -> Self {
        Self::new(raw_args.iter().map(|a| a.to_string()))
    }

    fn take_flag(&mut self, long_name: &str, short_name: Option<char>) -> bool {
        for raw_arg in &mut self.raw_args[..self.named_args_end] {
            let found = raw_arg.take_if(|raw_arg| {
                if raw_arg.starts_with("--") && &raw_arg[2..] == long_name {
                    true
                } else if short_name.is_some()
                    && raw_arg.starts_with('-')
                    && raw_arg.chars().count() == 2
                    && raw_arg.chars().nth(1) == short_name
                {
                    true
                } else {
                    false
                }
            });
            if found.is_some() {
                return true;
            }
        }
        false
    }

    pub fn version(&mut self) -> Version {
        Version::new(self)
    }

    pub fn help(&mut self) -> Help {
        Help::new(self)
    }

    pub fn output(self) -> Output {
        Output::new(self)
    }

    pub fn metadata(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

// TODO: move
// TODO: Rename to App (?)
#[derive(Debug)]
pub struct Metadata {
    pub app_name: &'static str,
    pub app_version: &'static str,
    pub app_description: Option<&'static str>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            app_name: env!("CARGO_PKG_NAME"),
            app_version: env!("CARGO_PKG_VERSION"),
            app_description: None,
        }
    }
}

// TODO: move
#[derive(Debug)]
pub struct Version<'a> {
    args: &'a mut CliArgs,
}

impl<'a> Version<'a> {
    fn new(args: &'a mut CliArgs) -> Self {
        Self { args }
    }

    pub fn is_present(self) -> bool {
        self.args.take_flag("version", None)
    }
}

#[derive(Debug)]
pub struct Help<'a> {
    args: &'a mut CliArgs,
    short_name: Option<char>,
}

impl<'a> Help<'a> {
    fn new(args: &'a mut CliArgs) -> Self {
        Self {
            args,
            short_name: None,
        }
    }

    pub fn short(mut self, name: char) -> Self {
        self.short_name = Some(name);
        self
    }

    pub fn is_present(self) -> bool {
        self.args.take_flag("help", self.short_name)
    }
}

// TODO: move
pub struct Output {
    args: CliArgs,
}

impl Output {
    fn new(args: CliArgs) -> Self {
        Self { args }
    }

    pub fn version_line(&self) -> String {
        format!(
            "{} {}",
            self.args.metadata.app_name, self.args.metadata.app_version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let mut args = CliArgs::from_slice(&["test", "run"]);
        assert!(!args.version().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--", "--version"]);
        assert!(!args.version().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--version"]);
        assert!(args.version().is_present());
        assert!(!args.version().is_present());

        args.metadata().app_name = "test";
        args.metadata().app_version = "0.0.1";
        assert_eq!(args.output().version_line(), "test 0.0.1");
    }

    #[test]
    fn help() {
        let mut args = CliArgs::from_slice(&["test", "run"]);
        assert!(!args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "--help"]);
        assert!(args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "-h"]);
        assert!(!args.help().is_present());

        let mut args = CliArgs::from_slice(&["test", "run", "-h"]);
        assert!(args.help().short('h').is_present());

        // TODO: help text
    }
}
