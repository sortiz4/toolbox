use clap::Parser;
use std::env;
use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::process::ExitCode;
use textwrap::fill;
use textwrap::Options as TextwrapOptions;
use textwrap::WordSeparator;
use textwrap::WordSplitter;
use textwrap::WrapAlgorithm;
use toolbox::cli::command::Arguments as CliArguments;
use toolbox::cli::command::Command as CliCommand;
use toolbox::cli::stream::Streams;
use toolbox::error::Error;
use toolbox::result::Result;

/// Wrap standard input at word boundaries.
#[derive(Parser)]
#[command(name = "wrap", disable_help_flag = true, disable_version_flag = true)]
struct Arguments {
    /// The line length to wrap.
    #[arg(short, long, default_value_t = 120)]
    length: usize,

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
        let mut input = String::new();
        (&self.streams.stdin).read_to_string(&mut input)?;

        let width = if self.arguments.length == 0 {
            50
        } else {
            self.arguments.length
        };

        let options = {
            TextwrapOptions::new(width)
                .break_words(false)
                .word_separator(WordSeparator::AsciiSpace)
                .word_splitter(WordSplitter::NoHyphenation)
                .wrap_algorithm(WrapAlgorithm::FirstFit)
        };

        write!(&self.streams.stdout, "{}", fill(&input, options))?;
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
