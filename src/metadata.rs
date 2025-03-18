use std::io::Write;

use crate::CliArgs;

#[derive(Debug)]
pub struct AppMetadata<'a, W> {
    args: &'a mut CliArgs<W>,
}

impl<'a, W: Write> AppMetadata<'a, W> {
    pub(crate) fn new(args: &'a mut CliArgs<W>) -> Self {
        Self { args }
    }

    pub fn version(self) -> AppVersion<'a, W> {
        AppVersion {
            args: self.args,
            consumed: false,
        }
    }
}

#[derive(Debug)]
pub struct AppVersion<'a, W: Write> {
    args: &'a mut CliArgs<W>,
    consumed: bool,
}

impl<'a, W: Write> Drop for AppVersion<'a, W> {
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
