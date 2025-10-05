use crate::error::ToroResult;

pub mod list;

pub trait Command: clap::Args {
    fn exec(self) -> ToroResult<()>;
}
