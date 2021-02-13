//! The Newt `Error` type.

use std::path::{Path, PathBuf};

/// Newt errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error parsing a configuration file.
    #[error(
        "Error in {} at line {line}: {kind}",
        .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| String::from("configuration"))
    )]
    Config {
        /// The line of the file that contains the error.
        line: usize,

        /// The path to the configuration file, if available.
        path: Option<PathBuf>,

        /// The kind of error.
        kind: ConfigErrorKind,
    },

    /// No notes directory was configured or could be found.
    #[error("No notes directory configured or found")]
    NoNotesDir,

    /// No editor was configured or could be found.
    #[error("No editor configured or found")]
    NoEditor,

    /// No pager program was configured or could be found.
    #[error("No pager configured or found")]
    NoPager,

    /// The user specified a file index that does not exist.
    #[error("No file with index {index}")]
    FileIndexOutOfRange {
        /// The provided, out-of-range index.
        index: usize,
    },

    /// The editor command could not be parsed or invoked.
    #[error(
        "Cannot invoke command `{}`{}",
        .command.display(),
        if let Some(err) = .source {
            format!(": {}", err)
        } else {
            String::new()
        },
    )]
    CannotInvoke {
        /// The offending command.
        command: PathBuf,

        /// The underlying OS error, if any.
        source: Option<std::io::Error>,
    },

    /// A system IO error.
    #[error("File IO error: {source}")]
    FileIo {
        /// The source IO error.
        #[from]
        source: std::io::Error,
    },
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Error::Config {
                    line: selfline,
                    path: selfpath,
                    kind: selfkind,
                },
                Error::Config {
                    line: otherline,
                    path: otherpath,
                    kind: otherkind,
                },
            ) => selfline == otherline && selfkind == otherkind && selfpath == otherpath,

            _ => false,
        }
    }
}

/// Newt configuration error kinds.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    /// An unrecognized configuration key was specified.
    #[error("unrecognized key {key:?}")]
    UnrecognizedKey {
        /// The unrecognized key.
        key: String,
    },

    /// An illegal or unexpected token was found.
    #[error("illegal token {token:?}")]
    IllegalToken {
        /// The illegal token.
        token: String,
    },

    /// The file ended unexpectedly.
    #[error("file ended unexpectedly")]
    UnexpectedEof,

    /// A string value was unterminated.
    #[error("missing '\"' character at end of string")]
    UnterminatedString,
}

impl ConfigErrorKind {
    /// Build an [`Error::Config`] from this `ConfigErrorKind`.
    pub fn at_line<P: AsRef<Path>>(self, line: usize, path: Option<P>) -> Error {
        Error::Config {
            line,
            path: path.map(|p| PathBuf::from(p.as_ref())),
            kind: self,
        }
    }
}

/// `Result` type specialized to Newt errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn cannot_invoke<S, O>(command: S, source: O) -> Error
where
    PathBuf: From<S>,
    O: Into<Option<std::io::Error>>,
{
    Error::CannotInvoke {
        command: PathBuf::from(command),
        source: source.into(),
    }
}

pub(crate) fn unrecognized_key<T, S>(key: S, line: usize) -> Result<T>
where
    String: From<S>,
{
    Err(Error::Config {
        line,
        path: None,
        kind: ConfigErrorKind::UnrecognizedKey {
            key: String::from(key),
        },
    })
}

pub(crate) fn illegal_token<T, S>(tok: S, line: usize) -> Result<T>
where
    String: From<S>,
{
    Err(Error::Config {
        line,
        path: None,
        kind: ConfigErrorKind::IllegalToken {
            token: String::from(tok),
        },
    })
}

pub(crate) fn unexpected_eof<T>(line: usize) -> Result<T> {
    Err(Error::Config {
        line,
        path: None,
        kind: ConfigErrorKind::UnexpectedEof,
    })
}

pub(crate) fn unterminated_string<T>(line: usize) -> Result<T> {
    Err(Error::Config {
        line,
        path: None,
        kind: ConfigErrorKind::UnterminatedString,
    })
}
