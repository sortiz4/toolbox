use clap::Parser;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::stream::Streams;
use toolbox::error::Error;
use toolbox::result::Result;

/// Remove Python cache directories and bytecode.
#[derive(Parser)]
#[command(name = "pyclean", disable_help_flag = true, disable_version_flag = true)]
struct Arguments {
    /// Roots to clean (default: current directory).
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    #[command(flatten)]
    cli: CliArguments,
}

struct Command {
    arguments: Arguments,
    streams: Streams,
}

struct Options<I = Vec<OsString>> {
    arguments: I,
    streams: Streams,
}

impl CliCommand for Command {
    type Arguments = Arguments;
    type Error = Error;

    fn main(&mut self) -> Result<()> {
        let clean = |path: &PathBuf| -> Result<()> {
            let command = {
                r"fd -HI --exclude=/System/Volumes/Data --exec-batch rm -rf ; -- (?:^__pycache__$|\.pyc$|\.pyo$)"
                    .split(' ')
                    .map(OsString::from)
                    .chain([path.as_os_str().to_owned()])
                    .collect::<Vec<_>>()
            };

            return self.run_process(&command, true);
        };

        return Self::run_batch(&self.arguments.paths, Self::has_disjoint_roots(&self.arguments.paths), clean);
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

fn main() -> ExitCode {
    let options = Options {
        arguments: env::args_os(),
        streams: Streams::default(),
    };

    if let Ok(mut command) = Command::try_from(options) {
        if let Ok(_) = command.run() {
            return ExitCode::SUCCESS;
        }
    }

    return ExitCode::FAILURE;
}
