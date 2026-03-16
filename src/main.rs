use clap::Parser;

use crate::commands::Command;
use crate::config::*;
use crate::interaction::FieldSelection;

mod commands;
mod config;
mod date;
mod error;
mod exec;
mod filter;
mod home;
mod interaction;
mod projects;
mod todotxt;


const NOTE_DIR: &str = "notes";


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

    /// Dump configuration
    Config(commands::config::ConfigCommand),

    /// Generate shell completions
    #[clap(hide = true)]
    Completions(commands::completions::CompletionsCommand),

    /// Mark task as completed
    Done(commands::done::DoneCommand),

    /// Set due date
    Due(commands::set::SetCommand),

    /// Execute a git command inside the data directory
    Git(commands::git::GitCommand),

    /// Initialize the data directory
    Init(commands::init::InitCommand),

    /// Generate man pages
    #[clap(hide = true)]
    Man(commands::man::ManCommand),

    /// Create a new task
    #[clap(alias("add"))]
    New(commands::new::NewCommand),

    /// Edit notes for a project
    Notes(commands::notes::NotesCommand),

    /// View projects
    #[clap(alias("projects"))]
    Project(commands::project::ProjectCommand),

    /// Change the priority of a task
    Prioritize(commands::set::SetCommand),

    /// Load and write back todo.txt file
    Rewrite(commands::rewrite::RewriteCommand),

    /// Set scheduled date
    Schedule(commands::set::SetCommand),

    /// Pull, rebase and push git repository
    Sync(commands::sync::SyncCommand),

    /// Update one or multiple tasks
    Update(commands::update::UpdateCommand),

    /// List all pending tasks
    View(commands::view::ViewCommand),
}


fn main() {
    let args = Args::parse();
    let config = error::resolve(home::load_config())
        .unwrap_or_default();

    use Subcommand::*;
    let result = match args.subcommand {
        Completions(cmd) => cmd.configure_exec(config),
        Config(cmd) => cmd.configure_exec(config),
        Done(cmd) => cmd.configure_exec(config),
        Due(cmd) => cmd.with_field(FieldSelection::Due).configure_exec(config),
        Edit(cmd) => cmd.configure_exec(config),
        Git(cmd) => cmd.configure_exec(config),
        Init(cmd) => cmd.configure_exec(config),
        Man(cmd) => cmd.configure_exec(config),
        New(cmd) => cmd.configure_exec(config),
        Notes(cmd) => cmd.configure_exec(config),
        Prioritize(cmd) => cmd.with_field(FieldSelection::Priority).configure_exec(config),
        Project(cmd) => cmd.configure_exec(config),
        Rewrite(cmd) => cmd.configure_exec(config),
        Schedule(cmd) => cmd.with_field(FieldSelection::Scheduled).configure_exec(config),
        Sync(cmd) => cmd.configure_exec(config),
        Update(cmd) => cmd.configure_exec(config),
        View(cmd) => cmd.configure_exec(config),
    };

    error::resolve(result)
}
