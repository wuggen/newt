//! Minimal note-taking CLI app, intended for quick thoughts and drafts.

#![warn(missing_docs)]

#[macro_use]
extern crate thiserror;

pub mod cli;
pub mod config;
pub mod error;
