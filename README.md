# Newt: Simple note-taking with minimal fuss.

A tool I made for myself after a few too many times wanting to just open up vim, write a
thing, and save it without having to worry about where to save it or what to call it.

## Building

Newt is written in Rust and has minimal dependencies. A simple `cargo build` should do the
trick.

## Usage

`newt --help`

```
newt 0.2.0

USAGE:
    newt [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Print verbose debugging output
    -y, --yes        Assume a 'yes' answer to all interactive prompts

OPTIONS:
    -f <config>                    Configuration file path
    -e, --editor <editor>          The editor command to invoke for editing notes
    -d, --notes-dir <notes-dir>    The directory in which to store notes

SUBCOMMANDS:
    cat          Print a note's contents to stdout
    edit         Edit a note in the configured editor
    help         Prints this message or the help of the given subcommand(s)
    list         List current notes
    new          Create a new note. Default if no other command is specified
    notes-dir    Print the canonicalized path to the configured notes directory
    rm           Delete a note from the notes directory
    view         View a note in the configured pager program
```

## Configuration

Newt searches for a configuration file in the following locations, in order of preference:

- The value passed to the `-f` command line option
- `$NEWT_CONFIG`
- `$XDG_CONFIG_HOME/newt/config`
- `$HOME/.config/newt/config`
- `$HOME/.newtrc`
- `/etc/newtrc`

If no configuration file is found, Newt will use default values for all options, as
detailed below.

The configuration file format is a simple sequence of keys and values.

- Keys and values are separated by any amount or kind of whitespace.
- Values may contain spaces if they are surrounded by double quotes (`"this is an
  example"`).
- Quoted strings may not span multiple lines.
- Within a quoted value, a literal double quote character, a newline, or a backslash can
  be inserted with the usual backslash escape sequences: `"examples: \" \n \\"`.
- Environment variables can be used with the syntax `$VAR` or `${VAR}`. They are expanded
  recursively.
- A dollar sign can be inserted without starting an evironment variable by doubling it:
  `$$`.
- A `#` character outside of a quoted value introduces a comment that extends to the end
  of the line.

The accepted configuration keys are as follows:

```
# The directory in which to save and look for notes.
# Defaults (in order of preference):
#   $NEWT_NOTES_DIR
#   $HOME/.newt
notes_dir $HOME/notes

# The program used for editing notes. It should accept a file path to edit as its first
# and only argument.
# Defaults (in order of preference):
#   $EDITOR
#   vim
#   vi
#   nano
editor emacs

# The program used to display a note with the "view" command. It should accept a file path
# to display as its first and only argument.
# Defaults (in order of preference):
#   $PAGER
#   less
#   more
#   cat
pager hexdump
```
