//! Command-line invocation and options.

use std::path::PathBuf;
use structopt::StructOpt;

/// Subcommand.
#[derive(Debug, Clone, StructOpt)]
pub enum Command {
    /// Create a new note. Default if no other command is specified.
    New {
        /// File name for the created note. Generates a unique name by default.
        #[structopt(short, long)]
        name: Option<String>,
    },

    /// List current notes.
    List,

    /// View a note in the configured pager program.
    View {
        /// File name of the target note.
        name: String,
    },

    /// Edit a note in the configured editor.
    Edit {
        /// File name of the target note.
        name: String,
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
    /// Configuration file.
    #[structopt(short = "f")]
    config: Option<PathBuf>,

    /// Subcommand.
    #[structopt(subcommand)]
    command: Option<Command>,
}
