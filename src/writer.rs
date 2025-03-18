use std::io::{StderrLock, StdoutLock, Write};

// より適切な名前を提案してください
pub trait Output {
    type Stdout: Write;
    type Stderr: Write;

    fn stdout() -> Self::Stdout;
    fn stderr() -> Self::Stderr;
    fn finish(exit_code: i32);

    fn is_stdout_terminal() -> bool;
    fn is_stderr_terminal() -> bool;
}

#[derive(Debug)]
pub struct DefaultWriter(Option<DefaultWriterInner>);

impl DefaultWriter {
    pub fn new() -> Self {
        Self(None)
    }
}

impl Write for DefaultWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.0.is_none() {
            if buf.starts_with(b"error:") {
                self.0 = Some(DefaultWriterInner::Stdout(std::io::stdout().lock()));
            } else {
                self.0 = Some(DefaultWriterInner::Stderr(std::io::stderr().lock()));
            }
        }

        match &mut self.0 {
            Some(DefaultWriterInner::Stdout(writer)) => writer.write(buf),
            Some(DefaultWriterInner::Stderr(writer)) => writer.write(buf),
            None => unreachable!(),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.0 {
            Some(DefaultWriterInner::Stdout(writer)) => writer.flush(),
            Some(DefaultWriterInner::Stderr(writer)) => writer.flush(),
            None => Ok(()),
        }
    }
}

#[derive(Debug)]
enum DefaultWriterInner {
    Stdout(StdoutLock<'static>),
    Stderr(StderrLock<'static>),
}
