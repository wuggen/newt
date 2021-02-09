//! Command-line invocation and options.

use crate::config::{self, Config};
use crate::error::*;
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
    println!("{:#?}", command);
    println!("{:#?}", config);

    println!(
        "Notes dir: {}",
        if let Some(path) = config.notes_dir() {
            path.display().to_string()
        } else {
            String::from("<not found>")
        }
    );
    println!(
        "Editor: {}",
        if let Some(command) = config.editor() {
            command.to_string_lossy().into_owned()
        } else {
            String::from("<not found>")
        }
    );

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
