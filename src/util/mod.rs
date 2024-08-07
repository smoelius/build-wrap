use anyhow::{ensure, Result};
use std::{env, ffi::OsString, fs::canonicalize, path::PathBuf, process::Command};

mod common;
pub use common::{exec_forwarding_output, ToUtf8};

#[cfg(test)]
#[allow(unused_imports)]
pub use common::assert_readme_contains_code_block;

pub const DEFAULT_LD: &str = "cc";

#[must_use]
pub fn cargo_build() -> Command {
    // smoelius: Respect `CARGO` environment variable, if set.
    let cargo = env::var_os("CARGO").unwrap_or(OsString::from("cargo"));

    let mut command = Command::new(cargo);
    command.arg("build");

    // smoelius: Show build script (e.g., wrapper) output.
    // See: https://github.com/rust-lang/cargo/issues/985#issuecomment-258311111
    command.arg("-vv");

    // smoelius: Show linker output.
    // See: https://stackoverflow.com/a/71866183
    // smoelius: Disabling this in the interest of reducing error message verbosity. Note that this
    // change affects only `build-wrap`'s building of wrapped build scripts.
    // command.env("RUSTC_LOG", "rustc_codegen_ssa::back::link=info");

    command
}

// smoelius: The present module is imported by tests/integration/util.rs. The next `allow` prevents
// a "function `which` is never used" warning in that module.
#[allow(dead_code)]
pub fn which(filename: &str) -> Result<PathBuf> {
    let mut command = Command::new("which");
    let output = command.arg(filename).output()?;
    ensure!(output.status.success(), "command failed: {command:?}");

    let stdout = std::str::from_utf8(&output.stdout)?;
    let path = canonicalize(stdout.trim_end())?;
    Ok(path)
}
