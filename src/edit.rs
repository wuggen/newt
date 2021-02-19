//! External command invocations.

use crate::config::Config;
use crate::error::*;
use crate::util::{env, sh};

use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Invoke the configured editor on the given path.
///
/// If a configured editor is found and the child process invocation is successful, returns the
/// exit status of the editor process. Otherwise returns an error.
pub fn edit_file<P: AsRef<Path>>(config: &Config, path: P) -> Result<ExitStatus> {
    let editor = config.editor()?;
    let interpolated = if let Some(e) = editor.to_str() {
        PathBuf::from(env::interpolate(e))
    } else {
        editor.clone()
    };

    let mut cmd = sh::command(&interpolated).ok_or_else(|| cannot_invoke(&editor, None))?;
    Ok(cmd
        .arg(path.as_ref())
        .status()
        .map_err(|err| cannot_invoke(&editor, err))?)
}

/// Invoke the configured editor on the given path, relative to the notes directory.
pub fn edit_note<P: AsRef<Path>>(config: &Config, path: P) -> Result<ExitStatus> {
    let mut full_path = config.notes_dir()?;
    full_path.push(path.as_ref());
    edit_file(config, full_path)
}

/// Invoke the configured pager on the given path, relative to the notes directory.
pub fn view_note<P: AsRef<Path>>(config: &Config, path: P) -> Result<ExitStatus> {
    let path = config.notes_dir()?.join(path.as_ref());
    let pager = config.pager()?;
    let interpolated = if let Some(p) = pager.to_str() {
        PathBuf::from(env::interpolate(p))
    } else {
        pager.clone()
    };

    let mut cmd = sh::command(&interpolated).ok_or_else(|| cannot_invoke(&pager, None))?;
    Ok(cmd
        .arg(&path)
        .status()
        .map_err(|err| cannot_invoke(&pager, err))?)
}
