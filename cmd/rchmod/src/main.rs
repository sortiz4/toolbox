use clap::Error as ClapError;
use clap::Parser;
use std::env;
use std::error::Error as StdError;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::io::Write;
use std::path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::result::Result as StdResult;
use std::sync::Mutex;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::stream::Streams;
use toolbox::error::Error as ToolboxError;
use toolbox::result::Result as ToolboxResult;
use walkdir::DirEntry;

type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
enum Error {
    Missing,
    Toolbox(ToolboxError),
}

/// Recursively change file and directory modes.
#[derive(Parser)]
#[command(name = "rchmod", disable_help_flag = true, disable_version_flag = true)]
struct Arguments {
    /// Mode to apply to files (octal or symbolic).
    #[arg(short, long)]
    file: Option<String>,

    /// Mode to apply to directories (octal or symbolic).
    #[arg(short, long)]
    dir: Option<String>,

    /// Explain changes without applying them.
    #[arg(short = 'D', long)]
    dry_run: bool,

    /// Suppress all prompts.
    #[arg(short, long)]
    suppress: bool,

    /// Explain what is being done.
    #[arg(short = 'V', long)]
    verbose: bool,

    /// Paths to modify.
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
        let output = Mutex::new(&self.streams.stdout);

        let change = |entry: &DirEntry| -> ToolboxResult<()> {
            let mode = if entry.file_type().is_dir() {
                self.arguments.dir.as_deref().unwrap_or("")
            } else {
                self.arguments.file.as_deref().unwrap_or("")
            };

            if mode.is_empty() {
                return Ok(());
            }

            let message = if self.arguments.dry_run {
                "will be changed"
            } else {
                let absolute = path::absolute(entry.path())?;

                let command = {
                    "chmod --"
                        .split(' ')
                        .map(OsString::from)
                        .chain([OsString::from(mode), absolute.into_os_string()])
                        .collect::<Vec<_>>()
                };

                self.run_process(&command, false).map_err(|error| ToolboxError::other(format!("chmod {:?}: {error}", entry.path())))?;
                "changed"
            };

            if self.arguments.dry_run || self.arguments.verbose {
                let mut stdout = output.lock().unwrap();
                writeln!(*stdout, "{:?}: mode {message}.", entry.path())?;
            }

            return Ok(());
        };

        if self.arguments.file.as_deref().unwrap_or("").is_empty() && self.arguments.dir.as_deref().unwrap_or("").is_empty() {
            return Err(Error::Missing);
        }

        let mut failures = Vec::new();

        for path in &self.arguments.paths {
            if path.is_absolute() && !self.arguments.suppress && !self.confirm(format!("{:?} is absolute", path))? {
                if self.arguments.verbose {
                    writeln!(&self.streams.stderr, "Skipped.")?;
                }

                continue;
            }

            let levels = match Self::levels(path) {
                Ok(levels) => levels,

                Err(error) => {
                    failures.push(error.into());
                    continue;
                },
            };

            for level in levels.iter().rev() {
                if let Err(error) = Self::run_batch(level, true, &change) {
                    failures.push(error);
                    break;
                }
            }
        }

        if failures.is_empty() {
            return Ok(());
        }

        return Err(ToolboxError::from(failures).into());
    }

    fn streams(&self) -> &Streams {
        return &self.streams;
    }

    fn arguments(&self) -> &CliArguments {
        return &self.arguments.cli;
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        return match self {
            Self::Missing => write!(fmt, "at least one of --file or --dir is required"),
            Self::Toolbox(err) => Display::fmt(err, fmt),
        };
    }
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        return Self::Toolbox(err.into());
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        return Self::Toolbox(err.into());
    }
}

impl From<ToolboxError> for Error {
    fn from(err: ToolboxError) -> Self {
        return Self::Toolbox(err);
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        return match self {
            Self::Missing => None,
            Self::Toolbox(err) => Some(err),
        };
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
