use std::fmt::Display;
use std::path::PathBuf;
use std::{io, process};

use colored::Colorize;

use crate::todotxt;

pub type ToroResult<T> = Result<T, ToroError>;

#[derive(Debug, thiserror::Error)]
pub enum ToroError {
    #[error("{0}: {1}")]
    NamedIOError(PathBuf, io::Error),

    #[error("{0}")]
    IOError(#[from] io::Error),

    #[error("Invalid syntax\n{0}")]
    SyntaxError(Box<pest::error::Error<todotxt::Rule>>),

    #[error("Invalid date \"{0}\"")]
    DateInputError(String),

    #[error("Data file not found")]
    DataFileNotFound(),
}

pub fn resolve<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            process::exit(1)
        },
    }
}

pub fn warn<T, E: Display>(result: Result<T, E>, default: T) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "Warning:".yellow(), e);
            default
        },
    }
}
