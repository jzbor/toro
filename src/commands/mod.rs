use crate::error::ToroResult;
use crate::Config;

pub mod completions;
pub mod done;
pub mod edit;
pub mod git;
pub mod man;
pub mod new;
pub mod rewrite;
pub mod sync;
pub mod update;
pub mod view;

pub trait Command: clap::Args {
    fn exec(&self) -> ToroResult<()>;

    fn config_mut(&mut self) -> &mut Config;

    fn configure_exec(&mut self, config: &Config) -> ToroResult<()> {
        self.config_mut().update_with_cmdline(config);
        self.exec()
    }
}
