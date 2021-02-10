//! File finding and editor invocation.

use crate::config::Config;
use crate::error::*;
use crate::util::{env, sh};

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Invoke the configured editor on the given path.
///
/// If a configured editor is found and the child process invocation is successful, returns the
/// exit status of the editor process. Otherwise returns an error.
pub fn edit_file<P: AsRef<Path>>(config: &Config, path: P) -> Result<ExitStatus> {
    let editor = config.editor().ok_or(Error::NoEditor)?;
    let interpolated = if let Some(e) = editor.to_str() {
        PathBuf::from(env::interpolate(e))
    } else {
        editor.clone()
    };

    let mut cmd = sh::command(&interpolated).ok_or_else(|| cannot_invoke(&editor))?;
    Ok(cmd
        .arg(path.as_ref())
        .status()
        .map_err(|_| cannot_invoke(&editor))?)
}

/// Invoke the configured editor on the given path, relative to the notes directory.
pub fn edit_note<P: AsRef<Path>>(config: &Config, path: P) -> Result<ExitStatus> {
    let mut full_path = config.notes_dir().ok_or(Error::NoNotesDir)?;
    full_path.push(path.as_ref());
    edit_file(config, full_path)
}

/// Compute a file name for a new note in the configured notes directory.
pub fn new_file_name(config: &Config) -> Result<PathBuf> {
    let notes_dir = config.notes_dir().ok_or(Error::NoNotesDir)?;
    let files = fs::read_dir(&notes_dir)?
        .map(|res| res.map(|dirent| dirent.file_name()))
        .collect::<Result<Vec<_>, _>>()?;

    let base = chrono::Local::today().format("%Y-%m-%d").to_string();
    let mut idx = 0;
    let name = loop {
        let name = format!("{}_{}.md", base, idx);
        if !files.contains(&OsString::from(&name)) {
            break name;
        } else {
            idx += 1;
        }
    };

    Ok(notes_dir.join(name))
}
