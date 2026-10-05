use std::env;
use std::env::consts;
use std::process::ExitCode;
use toolbox::cli::stream::Streams;
use toolbox::cmd::forward::Options;
use toolbox::cmd::forward::Command;

fn main() -> ExitCode {
    let Ok(executable) = env::current_exe() else {
        return ExitCode::FAILURE;
    };

    let options = Options {
        arguments: env::args_os(),
        program: {
            executable
                .with_file_name("lib")
                .join("grean-2.0.5/bin/grean")
                .with_extension(consts::EXE_EXTENSION)
        },
        streams: Streams::default(),
    };

    return Command::from(options).run().unwrap_or(ExitCode::FAILURE);
}
