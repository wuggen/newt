//! Command-line invocation and options.

use crate::config::{self, Config};
use crate::edit;
use crate::error::*;
use crate::notes_dir;
use crate::util;

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

    /// Print a note's contents to stdout.
    Cat {
        /// Index of the file, as displayed by the list command.
        index: usize,
    },

    /// Edit a note in the configured editor.
    Edit {
        /// Index of the file, as displayed by the list command.
        index: usize,
    },

    /// Print the canonicalized path to the configured notes directory.
    NotesDir,
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

fn new(config: &Config, name: Option<String>) -> Result<()> {
    let name = name
        .map(|n| Ok(PathBuf::from(n)))
        .unwrap_or_else(|| notes_dir::new_file_name(&config))?;
    let status = edit::edit_note(&config, &name)?;
    if !status.success() {
        eprintln!("Warning: editor process returned with status {}", status);
    }
    Ok(())
}

fn list(config: &Config) -> Result<()> {
    let files = notes_dir::list(config)?;
    let digits_space = util::digits(files.len()) + 1;

    let first_lines = files
        .iter()
        .map(|name| {
            let name_space = name.display().to_string().chars().count() + 3;
            notes_dir::first_line(config, name, 80 - name_space - digits_space)
        })
        .collect::<Result<Vec<_>>>()?;

    for (i, (name, line)) in files.iter().zip(first_lines.iter()).enumerate() {
        println!(
            "{} {} - {}",
            i,
            name.display(),
            line.as_deref().unwrap_or("<empty>")
        );
    }

    Ok(())
}

fn view(config: &Config, index: usize) -> Result<()> {
    let file = notes_dir::file_at_index(config, index)?;
    let status = edit::view_note(config, &file)?;
    if !status.success() {
        eprintln!("Warning: pager process returned with status {}", status);
    }
    Ok(())
}

fn cat(config: &Config, index: usize) -> Result<()> {
    let file = notes_dir::file_at_index(config, index)?;
    notes_dir::cat_file(config, file, &mut std::io::stdout())
}

fn edit(config: &Config, index: usize) -> Result<()> {
    let file = notes_dir::file_at_index(config, index)?;
    let status = edit::edit_note(config, &file)?;
    if !status.success() {
        eprintln!("Warning: editor process returned with status {}", status);
    }
    Ok(())
}

fn notes_dir(config: &Config) -> Result<()> {
    let path = config.notes_dir()?;
    println!("{}", path.canonicalize()?.display());
    Ok(())
}

/// Execute the given command with the given configuration.
pub fn execute(command: Command, config: Config) -> Result<()> {
    match command {
        Command::New { name } => new(&config, name),
        Command::List => list(&config),
        Command::View { index } => view(&config, index),
        Command::Cat { index } => cat(&config, index),
        Command::Edit { index } => edit(&config, index),
        Command::NotesDir => notes_dir(&config),
    }
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
