use std::ffi::OsStr;
use std::process::{self, Stdio};

use crate::error::{ToroError, ToroResult};

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

pub fn exec_quiet(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> ToroResult<()> {
    let status = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        Err(ToroError::ExternalCommandFailed(cmd.to_owned()))
    } else {
        Ok(())
    }
}
