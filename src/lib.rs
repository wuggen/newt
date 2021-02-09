//! Minimal note-taking CLI app, intended for quick thoughts and drafts.

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

macro_rules! dbg {
    ($($args:tt)*) => {
        crate::debug::dbg(format_args!($($args)*))
    }
}

pub(crate) mod debug;

pub mod cli;
pub mod config;
pub mod edit;
pub mod error;
