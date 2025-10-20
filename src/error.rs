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

    #[error("External command '{0}' failed")]
    ExternalCommandFailed(String),

    #[error("Environment variable '{0}' required, but not defined")]
    MissingEnvVar(&'static str),

    #[error("Invalid config file\n{0}")]
    ConfigSyntaxError(#[from] toml::de::Error),

    #[error("Unable to serialize config\n{0}")]
    ConfigSerializationError(#[from] toml::ser::Error),

    #[error("{0}")]
    ManError(String),

    #[error("{0}")]
    CompletionsError(String),

    #[error("{0}")]
    ReadlineError(#[from] rustyline::error::ReadlineError),

    #[error("Encountered EOF")]
    EofError(),
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
