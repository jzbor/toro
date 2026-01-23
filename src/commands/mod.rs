use crate::config::UpdatableConfig;
use crate::error::ToroResult;
use crate::Config;

pub mod completions;
pub mod config;
pub mod done;
pub mod edit;
pub mod git;
pub mod init;
pub mod man;
pub mod new;
pub mod notes;
pub mod rewrite;
pub mod set;
pub mod sync;
pub mod update;
pub mod view;

pub trait Command: clap::Args {
    fn exec(&self) -> ToroResult<()>;

    fn config_mut(&mut self) -> &mut Config;

    fn configure_exec(mut self, mut config: Config) -> ToroResult<()> {
        config.update_with_cmdline(self.config_mut());
        *self.config_mut() = config;
        self.exec()
    }
}
