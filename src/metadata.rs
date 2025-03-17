use crate::CliArgs;

#[derive(Debug)]
pub struct AppMetadata<'a> {
    args: &'a mut CliArgs,
}

impl<'a> AppMetadata<'a> {
    pub(crate) fn new(args: &'a mut CliArgs) -> Self {
        Self { args }
    }

    pub fn version(self) -> AppVersion<'a> {
        AppVersion {
            args: self.args,
            consumed: false,
        }
    }
}

#[derive(Debug)]
pub struct AppVersion<'a> {
    args: &'a mut CliArgs,
    consumed: bool,
}

impl<'a> Drop for AppVersion<'a> {
    fn drop(&mut self) {
        if self.consumed {
            return;
        }

        // let stdout = std::io::stdout();
        // let mut stdout = stdout.lock();
        // let _ = writeln!(
        //     stdout,
        //     "{} {}",
        //     env!("CARGO_PKG_NAME"),
        //     env!("CARGO_PKG_VERSION")
        // );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        //
    }
}
