//! Utilities for querying and managing the notes directory.

use crate::config::Config;
use crate::error::*;

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Get a sorted list of file names in the notes directory.
///
/// The elements of the returned vector are file names, rather than paths; that is, they are
/// paths relative to the notes directory.
pub fn list(config: &Config) -> Result<Vec<PathBuf>> {
    let notes_dir = config.notes_dir()?;
    let mut file_names = fs::read_dir(&notes_dir)?
        .map(|res| {
            res.map(|dirent| {
                let name = PathBuf::from(dirent.file_name());
                let path = notes_dir.join(&name);
                if let Ok(md) = fs::metadata(path) {
                    (name, Some(md))
                } else {
                    (name, None)
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    file_names.sort_by(|(name1, md1), (name2, md2)| {
        if let Some((md1, md2)) = md1.as_ref().zip(md2.as_ref()) {
            if let Some((t1, t2)) = md1.created().ok().zip(md2.created().ok()) {
                return t1.cmp(&t2);
            }
        }
        name1.cmp(name2)
    });

    Ok(file_names.into_iter().map(|(name, _)| name).collect())
}

/// Find a file name that does not yet exist in the configured note directory.
///
/// The returned `PathBuf` is a file name, rather than a path; it _is not_ prefixed by the path to
/// the notes directory.
pub fn new_file_name(config: &Config) -> Result<PathBuf> {
    let files = list(config)?;
    let base = chrono::Local::today().format("%Y-%m-%d").to_string();
    let mut idx = 0;
    Ok(loop {
        let name = PathBuf::from(format!("{}_{}.md", base, idx));
        if !files.contains(&name) {
            break name;
        } else {
            idx += 1;
        }
    })
}

/// Get the first non-empty line of the file at the given path relative to the notes directory.
///
/// A line is considered non-empty if it contains at least one non-whitespace character.
///
/// The returned line will be truncated if it is longer than `len` characters.
///
/// Returns `None` if the file contains no non-emtpy lines.
pub fn first_line<P: AsRef<Path>>(
    config: &Config,
    path: P,
    max_len: usize,
) -> Result<Option<String>> {
    let path = config.notes_dir()?.join(path);
    let mut lines = BufReader::new(File::open(path)?).lines();

    let first_line = lines
        .find(|res| match res {
            Err(_) => true,
            Ok(line) => line.chars().any(|c| !c.is_whitespace()),
        })
        .transpose()?;

    Ok(first_line.map(|line| {
        let len = line.chars().count();
        if len > max_len {
            format!("{}...", line.chars().take(max_len - 3).collect::<String>())
        } else {
            line
        }
    }))
}
