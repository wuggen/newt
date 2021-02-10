//! Command-line invocation and options.

use crate::config::{self, Config};
use crate::edit;
use crate::error::*;

use std::path::PathBuf;

use structopt::StructOpt;

/// Subcommand.
#[derive(Debug, Clone, StructOpt)]
pub enum Command {
    /// Create a new note. Default if no other command is specified.
    New {
        /// File name for the created note. Generates a unique name by default.
        name: Option<String>,
    },

    /// List current notes.
    List,

    /// View a note in the configured pager program.
    View {
        /// Index of the file, as displayed by the list command.
        index: usize,
    },

    /// Edit a note in the configured editor.
    Edit {
        /// Index of the file, as displayed by the list command.
        index: usize,
    },
}

impl Default for Command {
    fn default() -> Self {
        Command::New { name: None }
    }
}

/// Quick notetaking with minimal fuss.
#[derive(Debug, Clone, StructOpt)]
pub struct Options {
    /// Configuration file path.
    #[structopt(short = "f")]
    pub config: Option<PathBuf>,

    /// The directory in which to store notes.
    #[structopt(short = "d", long)]
    pub notes_dir: Option<PathBuf>,

    /// The editor command to invoke for editing notes.
    #[structopt(short, long)]
    pub editor: Option<PathBuf>,

    /// Print verbose debugging output.
    #[structopt(long, short)]
    pub verbose: bool,

    /// Subcommand.
    #[structopt(subcommand)]
    pub command: Option<Command>,
}

impl Options {
    /// Resolve the Newt configuration for these options.
    pub fn config(&self) -> Result<Config> {
        if let Some(path) = &self.config {
            config::read_config_file(path)
        } else {
            config::resolve()
        }
        .map(|config| {
            config
                .with_notes_dir(self.notes_dir.clone())
                .with_editor(self.editor.clone())
        })
    }
}

/// Execute the given command with the given configuration.
pub fn execute(command: Command, config: Config) -> Result<()> {
    match command {
        Command::New { name } => {
            let name = name
                .map(|n| Ok(PathBuf::from(n)))
                .unwrap_or_else(|| edit::new_file_name(&config))?;
            let status = edit::edit_note(&config, &name)?;
            if !status.success() {
                eprintln!("Warning: editor process returned with status {}", status);
            }
        }

        //Command::List => {
        //    todo!()
        //}

        c => {
            eprintln!("Command {:?} is not yet supported", c);
        }
    }

    Ok(())
}

/// Run the Newt CLI.
pub fn run() -> Result<()> {
    let options = Options::from_args();

    if options.verbose {
        crate::debug::verbose(true);
    }

    let config = options.config()?;
    execute(options.command.unwrap_or_default(), config)
}
