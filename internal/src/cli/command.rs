use super::stream::Streams;
use crate::error::Error;
use crate::result::Result;
use clap::Args as ClapArguments;
use clap::Command as ClapCommand;
use clap::CommandFactory;
use clap::Parser;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::process::Stdio;
use std::result::Result as StdResult;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ClapArguments)]
pub struct Arguments {
    /// Show this message.
    #[arg(short, long)]
    pub help: bool,

    /// Show the version.
    #[arg(short, long)]
    pub version: bool,
}

pub trait Command {
    type Arguments: Parser;
    type Error: StdError + From<IoError> + From<Error>;

    fn levels(root: &Path) -> IoResult<Vec<Vec<DirEntry>>> {
        let mut levels = Vec::new();

        for entry in WalkDir::new(root).follow_root_links(false).sort_by_file_name() {
            let entry = entry?;

            if !entry.file_type().is_dir() && !entry.file_type().is_file() {
                continue;
            }

            if levels.len() <= entry.depth() {
                levels.resize_with(entry.depth() + 1, Vec::new);
            }

            levels[entry.depth()].push(entry);
        }

        return Ok(levels);
    }

    fn run_batch<T, F>(items: &[T], parallelize: bool, run: F) -> Result<()> where T: Sync, F: Fn(&T) -> Result<()> + Sync + Send {
        let failures = if parallelize && items.len() > 1 {
            items.par_iter().filter_map(|item| run(item).err()).collect::<Vec<_>>()
        } else {
            items.iter().filter_map(|item| run(item).err()).collect::<Vec<_>>()
        };

        if failures.is_empty() {
            return Ok(());
        }

        return Err(Error::from(failures));
    }

    fn has_disjoint_roots(paths: &[PathBuf]) -> bool {
        let mut roots: Vec<PathBuf> = Vec::with_capacity(paths.len());

        for path in paths {
            let Ok(resolved) = fs::canonicalize(path) else {
                return false;
            };

            if roots.iter().any(|previous| resolved.starts_with(previous) || previous.starts_with(&resolved)) {
                return false;
            }

            roots.push(resolved);
        }

        return true;
    }

    fn run(&mut self) -> StdResult<(), Self::Error> {
        let mut run = || -> StdResult<(), Self::Error> {
            if self.arguments().help {
                let help = self.command().render_help();
                write!(&self.streams().stderr, "{}", help)?;
                return Ok(());
            }

            if self.arguments().version {
                let version = self.command().render_version();
                write!(&self.streams().stderr, "{}", version)?;
                return Ok(());
            }

            return self.main();
        };

        match run() {
            Ok(val) => {
                return Ok(val);
            },

            Err(err) => {
                writeln!(&self.streams().stderr, "Error: {}", err)?;
                return Err(err);
            },
        }
    }

    fn main(&mut self) -> StdResult<(), Self::Error>;

    fn confirm(&self, prompt: String) -> StdResult<bool, Self::Error> {
        // Keep later prompt answers in the underlying stream.
        let mut input = BufReader::with_capacity(1, &self.streams().stdin);

        loop {
            write!(&self.streams().stderr, "{prompt} - continue? [y/n] ")?;
            (&self.streams().stderr).flush()?;

            let mut answer = String::new();
            let count = input.read_line(&mut answer)?;

            match answer.trim().to_ascii_lowercase().as_str() {
                "y" => {
                    return Ok(true);
                },

                "n" => {
                    return Ok(false);
                },

                _ if count == 0 => {
                    return Err(IoError::from(ErrorKind::UnexpectedEof).into());
                },

                _ => {
                    // Not implemented
                },
            }
        }
    }

    fn run_process<S: AsRef<OsStr>>(&self, command: &[S], should_forward: bool) -> StdResult<(), Self::Error> {
        if command.is_empty() {
            return Err(IoError::new(ErrorKind::InvalidInput, "command cannot be empty").into());
        }

        let status = {
            StdCommand::new(command[0].as_ref())
                .args(&command[1..])
                .stdin(if should_forward { self.streams().stdin.to_stdio()? } else { Stdio::null() })
                .stdout(if should_forward { self.streams().stdout.to_stdio()? } else { Stdio::null() })
                .stderr(if should_forward { self.streams().stderr.to_stdio()? } else { Stdio::null() })
                .status()?
        };

        if status.success() {
            return Ok(());
        }

        return Err(Error::Process(command[0].as_ref().to_string_lossy().into_owned(), status).into());
    }

    fn streams(&self) -> &Streams;

    fn command(&self) -> ClapCommand {
        return Self::Arguments::command().version(VERSION);
    }

    fn arguments(&self) -> &Arguments;
}
