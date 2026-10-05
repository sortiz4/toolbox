use std::env;
use std::process::ExitCode;
use toolbox::cli::stream::Streams;
use toolbox::cmd::forward::Options;
use toolbox::cmd::forward::Command;

fn main() -> ExitCode {
    let options = Options {
        arguments: env::args_os(),
        program: "7zz",
        streams: Streams::default(),
    };

    return Command::from(options).run().unwrap_or(ExitCode::FAILURE);
}
