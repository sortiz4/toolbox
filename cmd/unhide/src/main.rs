use std::env;
use std::process::ExitCode;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::stream::Streams;
use toolbox::cmd::visibility::Options;
use toolbox::cmd::visibility::Command;
use toolbox::cmd::visibility::VisibilityMode;

fn main() -> ExitCode {
    let options = Options {
        arguments: env::args_os(),
        streams: Streams::default(),
        mode: VisibilityMode::Unhide,
    };

    if let Ok(mut command) = Command::try_from(options) {
        if let Ok(_) = command.run() {
            return ExitCode::SUCCESS;
        }
    }

    return ExitCode::FAILURE;
}
