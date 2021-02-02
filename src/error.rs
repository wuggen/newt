//! The Newt `Error` type.

/// Newt errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error parsing a configuration file.
    #[error("Error in configuration at line {line}: {kind}")]
    Config {
        /// The line of the file that contains the error.
        line: usize,

        /// The kind of error.
        kind: ConfigErrorKind,
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
                    kind: selfkind,
                },
                Error::Config {
                    line: otherline,
                    kind: otherkind,
                },
            ) => selfline == otherline && selfkind == otherkind,

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
    pub fn at_line(self, line: usize) -> Error {
        Error::Config { line, kind: self }
    }
}

/// `Result` type specialized to Newt errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn unrecognized_key<T, S>(key: S, line: usize) -> Result<T>
where
    String: From<S>,
{
    Err(Error::Config {
        line,
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
        kind: ConfigErrorKind::IllegalToken {
            token: String::from(tok),
        },
    })
}

pub(crate) fn unexpected_eof<T>(line: usize) -> Result<T> {
    Err(Error::Config {
        line,
        kind: ConfigErrorKind::UnexpectedEof,
    })
}

pub(crate) fn unterminated_string<T>(line: usize) -> Result<T> {
    Err(Error::Config {
        line,
        kind: ConfigErrorKind::UnterminatedString,
    })
}
