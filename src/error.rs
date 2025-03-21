#[derive(Debug)]
pub enum Error {
    //UnexpectedArgs { args: Vec<String> },
    // UnexpectedSubcommand
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     //Error::UnexpectedArgs { args } => todo!(),
        // }
        Ok(())
    }
}

impl std::error::Error for Error {}
