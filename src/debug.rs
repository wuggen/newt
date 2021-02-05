use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref VERBOSE: RwLock<bool> = RwLock::new(false);
}

/// Set the global debugging verbosity.
pub fn verbose(verbose: bool) {
    *VERBOSE.write().unwrap() = verbose;
}

/// Print a debug message, depending on the global verbosity setting.
pub fn dbg(args: std::fmt::Arguments<'_>) {
    if *VERBOSE.read().unwrap() {
        eprintln!("{}", args);
    }
}
