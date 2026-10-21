use clap::Parser;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::stream::Streams;
use toolbox::error::Error;
use toolbox::result::Result;
use walkdir::WalkDir;

/// Removes metadata created by native file explorers.
#[derive(Parser)]
#[command(name = "rmmeta", disable_help_flag = true, disable_version_flag = true)]
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

impl Command {
    // AppleDouble v2 header and entry layout: RFC 1740, appendices A and B
    fn is_apple_double(path: &Path) -> Result<bool> {
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();

        if size < 26 {
            return Ok(false);
        }

        let mut header = [0; 26];
        file.read_exact(&mut header)?;

        if header[..8] != [0x00, 0x05, 0x16, 0x07, 0x00, 0x02, 0x00, 0x00] {
            return Ok(false);
        }

        let count = u16::from_be_bytes([header[24], header[25]]);
        let table_end = 26 + u64::from(count) * 12;

        if count == 0 || table_end > size {
            return Ok(false);
        }

        for _ in 0..count {
            let mut entry = [0; 12];
            file.read_exact(&mut entry)?;

            let id = u32::from_be_bytes(entry[..4].try_into().unwrap());
            let offset = u64::from(u32::from_be_bytes(entry[4..8].try_into().unwrap()));
            let length = u64::from(u32::from_be_bytes(entry[8..].try_into().unwrap()));

            // AppleDouble must not contain the original file's data fork (ID 1)
            if id <= 1 || offset < table_end || offset + length > size {
                return Ok(false);
            }
        }

        return Ok(true);
    }
}

impl CliCommand for Command {
    type Arguments = Arguments;
    type Error = Error;

    fn main(&mut self) -> Result<()> {
        let clean = |path: &PathBuf| -> Result<()> {
            let mut failures = Vec::new();

            let entries = {
                WalkDir::new(path)
                    .follow_root_links(false)
                    .into_iter()
                    .filter_entry(|entry| entry.path() != Path::new("/System/Volumes/Data"))
            };

            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,

                    Err(error) => {
                        failures.push(Error::other(error.to_string()));
                        continue;
                    },
                };

                if !entry.file_type().is_file() {
                    continue;
                }

                let name = entry.file_name().to_string_lossy();
                let might_be_apple_double = name.starts_with("._");

                let is_metadata = {
                    name == ".DS_Store" ||
                    name == ".directory" ||
                    ["Thumbs.db", "ehthumbs.db", "ehthumbs_vista.db"].iter().any(|cache| name.eq_ignore_ascii_case(cache))
                };

                if !is_metadata && !might_be_apple_double {
                    continue;
                }

                let file = entry.path();

                let clean_entry = || -> Result<()> {
                    // Recheck the type before reading or removing a candidate
                    if !fs::symlink_metadata(file)?.file_type().is_file() {
                        return Ok(());
                    }

                    if might_be_apple_double && !Self::is_apple_double(file)? {
                        return Ok(());
                    }

                    fs::remove_file(file)?;
                    return Ok(());
                };

                if let Err(error) = clean_entry() {
                    failures.push(Error::other(format!("{}: {error}", file.display())));
                }
            }

            if failures.is_empty() {
                return Ok(());
            }

            return Err(Error::from(failures));
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
