use clap::Parser;
use std::env;
use std::env::consts;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::process::ExitCode;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::stream::Streams;
use toolbox::error::Error;
use toolbox::result::Result;

const COMMANDS: &[&str] = &[
    "docker",
    #[cfg(target_os = "linux")]
    "firewallctl",
    "hide",
    #[cfg(any(target_os = "linux", windows))]
    "open",
    "py",
    "pyclean",
    "python",
    "rchmod",
    #[cfg(windows)]
    "renet",
    "reviso",
    #[cfg(target_os = "macos")]
    "rmdss",
    "rmem",
    "sqlite",
    "unhide",
    "wrap",
];

/// Build all tools supported on the current platform.
#[derive(Parser)]
#[command(name = "build", disable_help_flag = true, disable_version_flag = true)]
struct Arguments {
    /// Output directory; binaries go under OS-ARCH.
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,

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
        let package = |name: &&str| -> String {
            return match *name {
                "firewallctl" | "open" | "renet" => format!("toolbox-{name}-{}", consts::OS),
                "rmdss" => String::from("toolbox-rmdss-darwin"),
                _ => format!("toolbox-{name}"),
            };
        };

        let packages = {
            COMMANDS
                .iter()
                .map(package)
                .collect::<Vec<_>>()
                .join(" -p ")
        };

        let command = {
            format!("cargo build --release --target-dir target -p {packages}")
                .split(' ')
                .map(OsString::from)
                .collect::<Vec<_>>()
        };

        let workspace = {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .ok_or_else(|| Error::other("missing workspace directory"))?
        };

        let output = workspace.join(&self.arguments.output).join(format!("{}-{}", consts::OS, consts::ARCH));
        fs::create_dir_all(&output)?;
        writeln!(&self.streams.stderr, "Building tools for {}/{}", consts::OS, consts::ARCH)?;

        let status = {
            StdCommand::new(&command[0])
                .current_dir(workspace)
                .args(&command[1..])
                .stdin(self.streams.stdin.to_stdio()?)
                .stdout(self.streams.stdout.to_stdio()?)
                .stderr(self.streams.stderr.to_stdio()?)
                .status()?
        };

        if !status.success() {
            return Err(Error::other(format!("cargo build failed with {status}")));
        }

        for name in COMMANDS {
            let filename = format!("{name}{}", consts::EXE_SUFFIX);
            fs::copy(workspace.join("target/release").join(&filename), output.join(filename))?;
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
