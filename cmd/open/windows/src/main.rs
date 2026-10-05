use std::env;
use std::process::ExitCode;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::stream::Streams;
use toolbox::cmd::open::Options;
use toolbox::cmd::open::Command;

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
