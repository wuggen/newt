//! File finding and editor invocation.

use crate::config::Config;
use crate::error::*;
use sha1::Sha1;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitStatus;

pub fn edit_file(config: Config, path: PathBuf) -> Result<ExitStatus> {
    let digest = if let Ok(mut file) = File::open(&path) {
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let mut sha = Sha1::new();
        sha.update(&contents);
        sha.digest()
    } else {
        Sha1::new().digest()
    };

    let editor = config.editor().ok_or(Error::NoEditor)?;
    todo!()
}
