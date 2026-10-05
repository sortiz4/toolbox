use crate::cli::stream::Streams;
use crate::result::Result;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::process::ExitCode;

pub struct Command {
    arguments: Vec<OsString>,
    program: PathBuf,
    streams: Streams,
}

pub struct Options<I = Vec<OsString>, P = PathBuf> {
    pub arguments: I,
    pub program: P,
    pub streams: Streams,
}

impl Command {
    pub fn run(&self) -> Result<ExitCode> {
        let status = {
            StdCommand::new(&self.program)
                .args(&self.arguments)
                .stdin(self.streams.stdin.to_stdio()?)
                .stdout(self.streams.stdout.to_stdio()?)
                .stderr(self.streams.stderr.to_stdio()?)
                .status()?
        };

        return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
    }
}

impl<I, P> From<Options<I, P>> for Command where I: IntoIterator, I::Item: Into<OsString>, P: Into<PathBuf> {
    fn from(options: Options<I, P>) -> Self {
        return Self {
            arguments: {
                options.arguments
                    .into_iter()
                    .skip(1)
                    .map(Into::into)
                    .collect()
            },
            program: options.program.into(),
            streams: options.streams,
        };
    }
}
