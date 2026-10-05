use clap::Error as ClapError;
use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::io::Error as IoError;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum Error {
    Clap(ClapError),
    Io(IoError),
    Message(String),
    Multiple(Vec<Error>),
    Process(String, ExitStatus),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Self::Clap(err) => {
                return Display::fmt(err, fmt);
            },

            Self::Io(err) => {
                return Display::fmt(err, fmt);
            },

            Self::Message(message) => {
                return write!(fmt, "{message}");
            },

            Self::Multiple(errors) => {
                for (index, err) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(fmt)?;
                    }

                    Display::fmt(err, fmt)?;
                }

                return Ok(());
            },

            Self::Process(program, status) => {
                return write!(fmt, "{program} exited with {status}");
            },
        }
    }
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        return Self::Message(message.into());
    }
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        return Self::Clap(err);
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        return Self::Io(err);
    }
}

impl From<Vec<Error>> for Error {
    fn from(errors: Vec<Error>) -> Self {
        return Self::Multiple(errors);
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        return match self {
            Self::Clap(err) => Some(err),
            Self::Io(err) => Some(err),
            Self::Message(_) | Self::Multiple(_) | Self::Process(_, _) => None,
        };
    }
}
