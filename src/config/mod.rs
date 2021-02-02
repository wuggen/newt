//! Configuration file definitions.

use crate::error::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

mod parse;

fn config_paths() -> impl Iterator<Item = PathBuf> {
    let mut num = 0;
    std::iter::from_fn(move || loop {
        match num {
            0 => {
                if let Some(mut base) = dirs::config_dir() {
                    base.push("newt/config");
                    return Some(base);
                }
            }

            1 => {
                if let Some(mut base) = dirs::home_dir() {
                    base.push(".newtrc");
                    return Some(base);
                }
            }

            _ => return None,
        }

        num += 1;
    })
}

fn find_conf_file() -> Option<PathBuf> {
    for path in config_paths() {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if metadata.is_file() {
                return Some(path);
            }
        }
    }

    None
}

/// Resolve the Newt configuration.
pub fn resolve_configuration() -> Result<Config> {
    if let Some(path) = find_conf_file() {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Config::from_str(&contents)
    } else {
        Ok(Config::default())
    }
}

/// Newt configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Config {
    notes_dir: Option<PathBuf>,
}

impl Config {
    /// The configured notes directory, if available.
    pub fn notes_dir(&self) -> Option<PathBuf> {
        self.notes_dir.clone().or_else(|| {
            dirs::home_dir().map(|mut base| {
                base.push(".newt");
                base
            })
        })
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

    const EMPTY_CONF: &str = "";
    const WS_CONF: &str = "    \n   \n";
    const COMMENT_CONF: &str = r#"
    # Here's a comment
    # And another one

    ## Some# weird-ass ## extra hashes
    "#;

    const NOTES_DIR_CONF: &str = "notes_dir ~/.notes\n";
    const DUPLICATE_NOTES_DIR_CONF: &str = r"
    notes_dir ~/.notes
    notes_dir ~/wait/no/this/one # Change it up
    ";

    const QUOTED_VALUE_CONF: &str = r#"
    notes_dir "~/My Documents/is this windows"
    "#;

    const QUOTED_KEY_CONF: &str = r#"
    "notes_dir" ~ # Not really sure WHY you'd do this but hey
    "#;

    const MISSING_VALUE_CONF: &str = "notes_dir # lol nope";

    const BAD_KEY_CONF: &str = r#"not_a_key "heya bish""#;

    #[test]
    fn empty() {
        assert_eq!(Config::from_str(EMPTY_CONF).unwrap(), Config::default());
    }

    #[test]
    fn whitespace() {
        assert_eq!(Config::from_str(WS_CONF).unwrap(), Config::default());
    }

    #[test]
    fn comments() {
        assert_eq!(Config::from_str(COMMENT_CONF).unwrap(), Config::default());
    }

    #[test]
    fn notes_dir() {
        let expected = Config {
            notes_dir: Some(PathBuf::from("~/.notes")),
        };
        assert_eq!(Config::from_str(NOTES_DIR_CONF).unwrap(), expected);
    }

    #[test]
    fn duplicate_keys() {
        let expected = Config {
            notes_dir: Some(PathBuf::from("~/wait/no/this/one")),
        };
        assert_eq!(
            Config::from_str(DUPLICATE_NOTES_DIR_CONF).unwrap(),
            expected
        );
    }

    #[test]
    fn quoted_value() {
        let expected = Config {
            notes_dir: Some(PathBuf::from("~/My Documents/is this windows")),
        };
        assert_eq!(Config::from_str(QUOTED_VALUE_CONF).unwrap(), expected);
    }

    #[test]
    fn quoted_key() {
        let expected = Config {
            notes_dir: Some(PathBuf::from("~")),
        };
        assert_eq!(Config::from_str(QUOTED_KEY_CONF).unwrap(), expected);
    }

    #[test]
    fn missing_value() {
        assert_eq!(Config::from_str(MISSING_VALUE_CONF), unexpected_eof(1),);
    }

    #[test]
    fn bad_key() {
        assert_eq!(
            Config::from_str(BAD_KEY_CONF),
            unrecognized_key("not_a_key", 1)
        );
    }
}
