use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::commands::Command;

mod home;
mod commands;
mod error;
mod todotxt;
mod exec;
mod filter;
mod interaction;

/// CLI Client to manage a todo.txt list
#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// List all pending tasks
    View(commands::view::ViewCommand),

    /// Open todo.txt file in an editor
    Edit(commands::edit::EditCommand),

    /// Load and write back todo.txt file
    Rewrite(commands::rewrite::RewriteCommand),

    /// Mark task as completed
    Done(commands::done::DoneCommand),
}

fn main() {
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        View(cmd) => cmd.exec(),
        Edit(cmd) => cmd.exec(),
        Rewrite(cmd) => cmd.exec(),
        Done(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
