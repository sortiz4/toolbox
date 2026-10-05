use crate::cli::command::Command as CliCommand;
use crate::cli::command::Arguments as CliArguments;
use crate::cli::stream::Streams;
use crate::cli::command::VERSION;
use crate::error::Error;
use crate::result::Result;
use clap::Command as ClapCommand;
use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
#[cfg(not(windows))]
use std::ffi::OsStr;
use std::ffi::OsString;
#[cfg(not(windows))]
use std::fs;
#[cfg(not(windows))]
use std::io::ErrorKind;
#[cfg(not(windows))]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::path;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub enum VisibilityMode {
    Hide,
    Unhide,
}

#[derive(Parser)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub struct Arguments {
    /// Files or directories to rename or mark hidden.
    #[arg(required_unless_present_any = ["help", "version"], num_args = 1..)]
    paths: Vec<PathBuf>,

    #[command(flatten)]
    cli: CliArguments,
}

pub struct Command {
    arguments: Arguments,
    streams: Streams,
    mode: VisibilityMode,
}

pub struct Options<I = Vec<OsString>> {
    pub arguments: I,
    pub streams: Streams,
    pub mode: VisibilityMode,
}

impl CliCommand for Command {
    type Arguments = Arguments;
    type Error = Error;

    fn main(&mut self) -> Result<()> {
        let failures = {
            self.arguments.paths
                .iter()
                .filter_map(|path| self.change(path).err())
                .collect::<Vec<_>>()
        };

        if failures.is_empty() {
            return Ok(());
        }

        return Err(Error::from(failures));
    }

    fn streams(&self) -> &Streams {
        return &self.streams;
    }

    fn command(&self) -> ClapCommand {
        return Self::parser(self.mode);
    }

    fn arguments(&self) -> &CliArguments {
        return &self.arguments.cli;
    }
}

impl Command {
    #[cfg(windows)]
    fn change(&self, path: &Path) -> Result<()> {
        let flag = if matches!(self.mode, VisibilityMode::Hide) {
            "+h"
        } else {
            "-h"
        };

        let absolute = path::absolute(path)?;

        let command = {
            format!("attrib {flag}")
                .split(' ')
                .map(OsString::from)
                .chain([absolute.into_os_string()])
                .collect::<Vec<_>>()
        };

        return self.run_process(&command, false);
    }

    #[cfg(not(windows))]
    fn change(&self, path: &Path) -> Result<()> {
        let base = path.file_name().ok_or_else(|| Error::other(format!("cannot rename {:?}", path)))?;
        let bytes = base.as_bytes();

        if bytes == b"." || bytes == b".." {
            return Err(Error::other(format!("cannot rename {:?}", path)));
        }

        fs::symlink_metadata(path)?;

        let name = if matches!(self.mode, VisibilityMode::Hide) {
            if bytes.starts_with(b".") {
                return Ok(());
            }

            let mut value = Vec::with_capacity(bytes.len() + 1);
            value.push(b'.');
            value.extend_from_slice(bytes);
            value
        } else {
            if !bytes.starts_with(b".") {
                return Ok(());
            }

            let value = &bytes[1..];

            if value.is_empty() || value == b"." || value == b".." {
                return Err(Error::other(format!("cannot unhide {:?}", path)));
            }

            value.to_vec()
        };

        let target = path.with_file_name(OsStr::from_bytes(&name));

        match fs::symlink_metadata(&target) {
            Ok(_) => {
                return Err(Error::other(format!("destination {:?} already exists", target)));
            },

            Err(error) if error.kind() == ErrorKind::NotFound => {
                // Not implemented
            },

            Err(error) => {
                return Err(error.into());
            },
        }

        fs::rename(path, target)?;
        return Ok(());
    }

    fn parser(mode: VisibilityMode) -> ClapCommand {
        let (name, about) = match mode {
            VisibilityMode::Hide => ("hide", "Hide files and directories."),
            VisibilityMode::Unhide => ("unhide", "Unhide files and directories."),
        };

        return {
            Arguments::command()
                .name(name)
                .about(about)
                .version(VERSION)
        };
    }
}

impl<I> TryFrom<Options<I>> for Command where I: IntoIterator, I::Item: Into<OsString> + Clone {
    type Error = Error;

    fn try_from(options: Options<I>) -> Result<Self> {
        let matches = {
            Self::parser(options.mode)
                .try_get_matches_from(options.arguments)?
        };

        return Ok(
            Self {
                arguments: Arguments::from_arg_matches(&matches)?,
                streams: options.streams,
                mode: options.mode,
            }
        );
    }
}
