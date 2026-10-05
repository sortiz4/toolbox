use crate::cli::command::Command as CliCommand;
use crate::cli::command::Arguments as CliArguments;
use crate::cli::stream::Streams;
use crate::error::Error;
use crate::result::Result;
use clap::Parser;
use std::ffi::OsString;
use std::path;
use std::path::PathBuf;

/// Open paths with the platform default application.
#[derive(Parser)]
#[command(name = "open", disable_help_flag = true, disable_version_flag = true)]
pub struct Arguments {
    /// Paths to open (default: current directory).
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    #[command(flatten)]
    cli: CliArguments,
}

pub struct Command {
    arguments: Arguments,
    streams: Streams,
}

pub struct Options<I = Vec<OsString>> {
    pub arguments: I,
    pub streams: Streams,
}

impl CliCommand for Command {
    type Arguments = Arguments;
    type Error = Error;

    fn main(&mut self) -> Result<()> {
        for path in &self.arguments.paths {
            open::that(path::absolute(path)?)?;
        }

        return Ok(());
    }

    fn streams(&self) -> &Streams {
        return &self.streams;
    }

    fn arguments(&self) -> &CliArguments {
        return &self.arguments.cli;
    }
}

impl<I> TryFrom<Options<I>> for Command where I: IntoIterator, I::Item: Into<OsString> + Clone {
    type Error = Error;

    fn try_from(options: Options<I>) -> Result<Self> {
        return Ok(
            Self {
                arguments: Arguments::try_parse_from(options.arguments)?,
                streams: options.streams,
            }
        );
    }
}
