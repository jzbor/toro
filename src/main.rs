use clap::Parser;

use crate::commands::Command;
use crate::filter::ColumnSelector;

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

    /// Generate man pages
    #[clap(hide = true)]
    Man(commands::man::ManCommand),

    /// Generate shell completions
    #[clap(hide = true)]
    Completions(commands::completions::CompletionsCommand),
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Config {
    columns: ColumnSelector,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            columns: ColumnSelector::default(),
        }
    }
}


fn main() {
    let args = Args::parse();
    let config = error::resolve(home::load_config())
        .unwrap_or_default();

    use Subcommand::*;
    let result = match args.subcommand {
        View(cmd) => cmd.exec(config),
        Edit(cmd) => cmd.exec(config),
        Rewrite(cmd) => cmd.exec(config),
        Done(cmd) => cmd.exec(config),
        Man(cmd) => cmd.exec(config),
        Completions(cmd) => cmd.exec(config),
    };

    error::resolve(result)
}
