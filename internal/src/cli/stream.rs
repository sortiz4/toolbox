use std::fs::File;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::process::Stdio;

pub enum Stream {
    File(File),
    Null,
    Stdin,
    Stdout,
    Stderr,
}

pub struct Streams {
    pub stdin: Stream,
    pub stdout: Stream,
    pub stderr: Stream,
}

impl Default for Streams {
    fn default() -> Self {
        return Self {
            stdin: Stream::Stdin,
            stdout: Stream::Stdout,
            stderr: Stream::Stderr,
        };
    }
}

impl From<File> for Stream {
    fn from(file: File) -> Self {
        return Self::File(file);
    }
}

impl Read for &Stream {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        return match self {
            Stream::File(file) => (&*file).read(buffer),
            Stream::Null => Ok(0),
            Stream::Stdin => io::stdin().read(buffer),
            _ => Err(Error::new(ErrorKind::InvalidInput, "stream is not readable")),
        };
    }
}

impl Stream {
    pub fn to_stdio(&self) -> Result<Stdio> {
        return match self {
            Self::File(file) => Ok(file.try_clone()?.into()),
            Self::Null => Ok(Stdio::null()),
            Self::Stdin => Ok(Stdio::inherit()),
            Self::Stdout => Ok(io::stdout().into()),
            Self::Stderr => Ok(io::stderr().into()),
        };
    }
}

impl Write for &Stream {
    fn flush(&mut self) -> Result<()> {
        return match self {
            Stream::File(file) => (&*file).flush(),
            Stream::Null => Ok(()),
            Stream::Stdout => io::stdout().flush(),
            Stream::Stderr => io::stderr().flush(),
            _ => Err(Error::new(ErrorKind::InvalidInput, "stream is not writable")),
        };
    }

    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        return match self {
            Stream::File(file) => (&*file).write(buffer),
            Stream::Null => Ok(buffer.len()),
            Stream::Stdout => io::stdout().write(buffer),
            Stream::Stderr => io::stderr().write(buffer),
            _ => Err(Error::new(ErrorKind::InvalidInput, "stream is not writable")),
        };
    }
}
