use std::io::Write;

#[derive(Debug)]
pub struct CliArgs {
    raw_args: Vec<Option<String>>,
    show_help: bool,
    next_arg_index: Option<usize>,
}

impl CliArgs {
    pub fn new() -> Self {
        Self::with_raw_args(std::env::args().skip(1))
    }

    fn with_raw_args<I, T>(raw_args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let raw_args = raw_args
            .into_iter()
            .map(|a| Some(a.into()))
            .collect::<Vec<_>>();
        Self {
            raw_args,
            show_help: false,
            next_arg_index: None,
        }
    }

    // TODO: app()
    pub fn app_version(&mut self) -> AppVersion {
        AppVersion::new(self)
    }

    // TODO: help
}

// TODO: move
#[derive(Debug)]
pub struct AppVersion<'a> {
    args: &'a mut CliArgs,
    consumed: bool,
}

impl<'a> AppVersion<'a> {
    fn new(args: &'a mut CliArgs) -> Self {
        Self {
            args,
            consumed: false,
        }
    }
}

impl<'a> Drop for AppVersion<'a> {
    fn drop(&mut self) {
        if self.consumed {
            return;
        }

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        let _ = writeln!(
            stdout,
            "{} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
}
