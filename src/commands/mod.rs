use crate::error::ToroResult;

pub mod view;
pub mod edit;
pub mod rewrite;
pub mod done;

pub trait Command: clap::Args {
    fn exec(self) -> ToroResult<()>;
}
