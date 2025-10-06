use crate::error::ToroResult;
use crate::Config;

pub mod completions;
pub mod done;
pub mod edit;
pub mod man;
pub mod rewrite;
pub mod view;

pub trait Command: clap::Args {
    fn exec(self, config: Config) -> ToroResult<()>;
}
