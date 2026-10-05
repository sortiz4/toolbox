use clap::Parser;
use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::stream::Streams;
use toolbox::error::Error;
use toolbox::result::Result;
use walkdir::DirEntry;

/// Recursively remove empty directories.
#[derive(Parser)]
#[command(name = "rmem", disable_help_flag = true, disable_version_flag = true)]
struct Arguments {
    /// Do not remove directories; show currently empty candidates.
    #[arg(short, long)]
    dry_run: bool,

    /// Suppress all prompts.
    #[arg(short, long)]
    suppress: bool,

    /// Explain what is being done.
    #[arg(short = 'V', long)]
    verbose: bool,

    /// Roots to clean (the roots themselves are preserved).
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
        let output = Mutex::new(&self.streams.stderr);

        let roots = {
            self.arguments.paths
                .iter()
                .filter_map(|path| fs::canonicalize(path).ok())
                .collect::<HashSet<_>>()
        };

        let remove = |entry: &DirEntry| -> Result<()> {
            if !entry.file_type().is_dir() {
                return Ok(());
            }

            let mut contents = match fs::read_dir(entry.path()) {
                Ok(contents) => {
                    contents
                },

                Err(error) if error.kind() == ErrorKind::NotFound => {
                    return Ok(());
                },

                Err(error) => {
                    return Err(error.into());
                },
            };

            if let Some(child) = contents.next() {
                child?;
                return Ok(());
            }

            if roots.contains(&fs::canonicalize(entry.path())?) {
                return Ok(());
            }

            let message = if self.arguments.dry_run {
                "Will remove"
            } else {
                fs::remove_dir(entry.path()).map_err(|error| Error::other(format!("remove {:?}: {error}", entry.path())))?;
                "Removed"
            };

            if self.arguments.dry_run || self.arguments.verbose {
                let mut stderr = output.lock().unwrap();
                writeln!(*stderr, "{message}: {}", entry.path().display())?;
            }

            return Ok(());
        };

        let mut failures = Vec::new();

        for root in &self.arguments.paths {
            if root.is_absolute() && !self.arguments.suppress && !self.confirm(format!("{:?} is absolute", root))? {
                if self.arguments.verbose {
                    writeln!(&self.streams.stderr, "Skipped.")?;
                }

                continue;
            }

            let levels = match Self::levels(root) {
                Ok(levels) => {
                    levels
                },

                Err(error) => {
                    failures.push(error.into());
                    continue;
                },
            };

            for level in levels.iter().skip(1).rev() {
                if let Err(error) = Self::run_batch(level, true, &remove) {
                    failures.push(error);
                }
            }
        }

        if failures.is_empty() {
            return Ok(());
        }

        return Err(Error::from(failures));
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
