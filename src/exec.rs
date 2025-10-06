use std::collections::VecDeque;
use std::ffi::OsStr;
use std::process::{self, Stdio};

use crate::error::{ToroError, ToroResult};
use crate::home;

pub fn exec(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> ToroResult<()> {
    let status = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        Err(ToroError::ExternalCommandFailed(cmd.to_owned()))
    } else {
        Ok(())
    }
}
