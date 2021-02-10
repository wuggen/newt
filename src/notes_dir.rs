//! Utilities for querying and managing the notes directory.

use crate::config::Config;
use crate::error::*;

use std::fs;
use std::path::PathBuf;

/// Get a sorted list of file names in the notes directory.
///
/// The elements of the returned vector are file names, rather than paths; that is, they are
/// paths relative to the notes directory.
pub fn list(config: &Config) -> Result<Vec<(PathBuf, String)>> {
    let notes_dir = config.notes_dir().ok_or(Error::NoNotesDir)?;
    let mut file_names = fs::read_dir(&notes_dir)?
        .map(|res| res.map(|dirent| PathBuf::from(dirent.file_name())))
        .collect::<Result<Vec<_>, _>>()?;
    file_names.sort();
}
