use compile::Error as CompileError;
use path::PathBuf;
use std::error::Error as StdError;
use std::{fmt, io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Rendering Errors
    NotInjectable(PathBuf),
    NotIterable(PathBuf),
    TemplateNotFound(String),
    Undefined(PathBuf),
    // Other Errors
    Compile(CompileError),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<CompileError> for Error {
    fn from(e: CompileError) -> Error {
        Error::Compile(e)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use Error::*;

        match self {
            NotInjectable(_) => "variable not injectable",
            NotIterable(_) => "variable not iterable",
            TemplateNotFound(_) => "template not found",
            Undefined(_) => "variable undefined",
            Compile(ref error) => error.description(),
            Io(ref error) => error.description(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            NotInjectable(ref path) => write!(f, "variable '{}' not injectable", path),
            NotIterable(ref path) => write!(f, "variable '{}' not iterable", path),
            TemplateNotFound(ref name) => write!(f, "template '{}' not found", name),
            Undefined(ref path) => write!(f, "variable '{}' undefined", path),
            Compile(ref error) => error.fmt(f),
            Io(ref error) => error.fmt(f),
        }
    }
}
