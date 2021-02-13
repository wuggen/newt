//! Configuration file definitions.

use crate::error::*;
use crate::util::env;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod parse;

#[cfg(not(debug_assertions))]
const CONFIG_PATHS: &[&str] = &[
    "$NEWT_CONFIG",
    "$XDG_CONFIG_HOME/newt/config",
    "$HOME/.config/newt/config",
    "$HOME/.newtrc",
    "/etc/newtrc",
];

#[cfg(debug_assertions)]
const CONFIG_PATHS: &[&str] = &[
    "$NEWT_CONFIG",
    concat!(env!("CARGO_MANIFEST_DIR"), "/newtrc"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/config"),
    "./newtrc",
    "./config",
];

#[cfg(not(debug_assertions))]
const NOTES_PATHS: &[&str] = &["$NEWT_NOTES_DIR", "$HOME/.newt"];

#[cfg(debug_assertions)]
const NOTES_PATHS: &[&str] = &[
    "$NEWT_NOTES_DIR",
    concat!(env!("CARGO_MANIFEST_DIR"), "/notes"),
    "./notes",
];

const EDITORS: &[&str] = &["$EDITOR", "vim", "vi", "nano"];

const PAGERS: &[&str] = &["$PAGER", "less", "more", "cat"];

fn find_conf_file() -> Option<PathBuf> {
    for path in CONFIG_PATHS.iter().map(env::interpolate).map(PathBuf::from) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if metadata.is_file() {
                dbg!("Using configuration file {}", path.display());
                return Some(path);
            }
        }
    }

    dbg!("No configuration file found, using default config");
    None
}

/// Resolve the Newt configuration from the runtime environment.
pub fn resolve() -> Result<Config> {
    if let Some(path) = find_conf_file() {
        read_config_file(path)
    } else {
        Ok(Config::default())
    }
}

/// Read the Newt configuration from the given file.
pub fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = PathBuf::from(path.as_ref());
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Config::from_str(&contents).map_err(|err| match err {
        Error::Config { line, kind, .. } => Error::Config {
            line,
            kind,
            path: Some(path),
        },
        e => e,
    })
}

/// Newt configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Config {
    notes_dir: Option<PathBuf>,
    editor: Option<PathBuf>,
    pager: Option<PathBuf>,
}

impl Config {
    /// The configured notes directory, if available.
    pub fn notes_dir(&self) -> Result<PathBuf> {
        self.notes_dir
            .clone()
            .or_else(|| {
                NOTES_PATHS
                    .iter()
                    .map(env::interpolate)
                    .map(PathBuf::from)
                    .find(|path| {
                        if let Ok(md) = std::fs::metadata(path) {
                            if md.is_dir() {
                                dbg!("Using notes directory {}", path.display());
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
            })
            .ok_or(Error::NoNotesDir)
    }

    /// The configured editor command, if available.
    pub fn editor(&self) -> Result<PathBuf> {
        self.editor
            .clone()
            .or_else(|| {
                EDITORS
                    .iter()
                    .map(env::interpolate)
                    .map(PathBuf::from)
                    .find(|command| env::search_path(&command).is_some())
            })
            .ok_or(Error::NoEditor)
    }

    /// The configured pager command, if available.
    pub fn pager(&self) -> Result<PathBuf> {
        self.pager
            .clone()
            .or_else(|| {
                PAGERS
                    .iter()
                    .map(env::interpolate)
                    .map(PathBuf::from)
                    .find(|command| env::search_path(&command).is_some())
            })
            .ok_or(Error::NoPager)
    }
}

impl Config {
    /// Set the notes dir on this `Config`.
    pub fn with_notes_dir<O: Into<Option<PathBuf>>>(self, notes_dir: O) -> Self {
        Config {
            notes_dir: notes_dir.into().or(self.notes_dir),
            ..self
        }
    }

    /// Set the editor on this `Config`.
    pub fn with_editor<O: Into<Option<PathBuf>>>(self, editor: O) -> Self {
        Config {
            editor: editor.into().or(self.editor),
            ..self
        }
    }

    /// Set the pager on this `Config`.
    pub fn with_pager<O: Into<Option<PathBuf>>>(self, pager: O) -> Self {
        Config {
            pager: pager.into().or(self.pager),
            ..self
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(contents: &str) -> Result<Config> {
        let mut lexer = parse::Lexer::new(contents.chars());
        let mut config = Config::default();

        while let Some(tok) = lexer.scan()? {
            match tok.as_str() {
                "notes_dir" => {
                    if let Some(path) = lexer.scan()? {
                        config.notes_dir = Some(PathBuf::from(path));
                    } else {
                        return unexpected_eof(lexer.line());
                    }
                }

                "editor" => {
                    if let Some(command) = lexer.scan()? {
                        config.editor = Some(PathBuf::from(command));
                    } else {
                        return unexpected_eof(lexer.line());
                    }
                }

                "pager" => {
                    if let Some(command) = lexer.scan()? {
                        config.pager = Some(PathBuf::from(command));
                    } else {
                        return unexpected_eof(lexer.line());
                    }
                }

                s => return unrecognized_key(s, lexer.line()),
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn empty() {
        let conf = "";
        assert_eq!(Config::from_str(conf).unwrap(), Config::default());
    }

    #[test]
    fn whitespace() {
        let conf = "    \n   \n";
        assert_eq!(Config::from_str(conf).unwrap(), Config::default());
    }

    #[test]
    fn comments() {
        let conf = r#"#Here's a comment
# And another one

## Some# weird-add ## extra hashes
"#;
        assert_eq!(Config::from_str(conf).unwrap(), Config::default());
    }

    #[test]
    fn notes_dir() {
        let conf = "notes_dir ~/.notes\n";
        let expected = Config::default().with_notes_dir(PathBuf::from("~/.notes"));
        assert_eq!(Config::from_str(conf).unwrap(), expected);
    }

    #[test]
    fn editor() {
        let conf = "editor /usr/bin/fibbldee\n";
        let expected = Config::default().with_editor(PathBuf::from("/usr/bin/fibbldee"));
        assert_eq!(Config::from_str(conf).unwrap(), expected);
    }

    #[test]
    fn duplicate_keys() {
        let conf = r"notes_dir ~/.notes
notes_dir ~/wait/no/this/one # Change it up
";
        let expected = Config::default().with_notes_dir(PathBuf::from("~/wait/no/this/one"));
        assert_eq!(Config::from_str(conf).unwrap(), expected);
    }

    #[test]
    fn quoted_value() {
        let conf = r#"notes_dir "~/My Documents/is this windows""#;
        let expected =
            Config::default().with_notes_dir(PathBuf::from("~/My Documents/is this windows"));
        assert_eq!(Config::from_str(conf).unwrap(), expected);
    }

    #[test]
    fn quoted_key() {
        let conf = r#""notes_dir" ~ # Not really sure WHY you'd do this but hey"#;
        let expected = Config::default().with_notes_dir(PathBuf::from("~"));
        assert_eq!(Config::from_str(conf).unwrap(), expected);
    }

    #[test]
    fn missing_value() {
        let conf = "notes_dir # lol nope";
        assert_eq!(Config::from_str(conf), unexpected_eof(1));
    }

    #[test]
    fn bad_key() {
        let conf = r#"not_a_key "heya bish""#;
        assert_eq!(Config::from_str(conf), unrecognized_key("not_a_key", 1));
    }
}
