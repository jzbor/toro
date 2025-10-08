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
    /// Open todo.txt file in an editor
    Edit(commands::edit::EditCommand),

    /// Generate shell completions
    #[clap(hide = true)]
    Completions(commands::completions::CompletionsCommand),

    /// Mark task as completed
    Done(commands::done::DoneCommand),

    /// Execute a git command inside the data directory
    Git(commands::git::GitCommand),

    /// Generate man pages
    #[clap(hide = true)]
    Man(commands::man::ManCommand),

    /// Create a new task
    #[clap(alias("add"))]
    New(commands::new::NewCommand),

    /// Load and write back todo.txt file
    Rewrite(commands::rewrite::RewriteCommand),

    /// Pull, rebase and push git repository
    Sync(commands::sync::SyncCommand),

    /// Update one or multiple tasks
    Update(commands::update::UpdateCommand),

    /// List all pending tasks
    View(commands::view::ViewCommand),
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Config {
    pub columns: ColumnSelector,

    pub git: GitConfig,
}

#[derive(serde::Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]
struct GitConfig {
    pub auto_commit: bool,

    pub auto_sync: bool,
}



impl Default for Config {
    fn default() -> Self {
        Config {
            columns: ColumnSelector::default(),
            git: GitConfig {
                auto_commit: false,
                auto_sync: false,
            }
        }
    }
}


fn main() {
    let args = Args::parse();
    let config = error::resolve(home::load_config())
        .unwrap_or_default();

    use Subcommand::*;
    let result = match args.subcommand {
        Completions(cmd) => cmd.exec(config),
        Done(cmd) => cmd.exec(config),
        Edit(cmd) => cmd.exec(config),
        Git(cmd) => cmd.exec(config),
        Man(cmd) => cmd.exec(config),
        New(cmd) => cmd.exec(config),
        Rewrite(cmd) => cmd.exec(config),
        Sync(cmd) => cmd.exec(config),
        Update(cmd) => cmd.exec(config),
        View(cmd) => cmd.exec(config),
    };

    error::resolve(result)
}
