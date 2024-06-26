//! This file must remain in the tests folder to ensure that `CARGO_BIN_EXE_build-wrap` is defined
//! at compile time. See [Environment variables Cargo sets for crates].
//!
//! [Environment variables Cargo sets for crates]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates

// smoelius: Use this module with `pub` to avoid "unused ..." warnings.
// See: https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259/2

use anyhow::Result;
use cargo_metadata::{Metadata, MetadataCommand};
use once_cell::sync::Lazy;
use std::{
    env,
    fs::{copy, create_dir, write, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};
use tempfile::{tempdir_in, TempDir};

#[path = "../src/util/mod.rs"]
mod main_util;
pub use main_util::*;

#[ctor::ctor]
fn initialize() {
    env::set_var("CARGO_TERM_COLOR", "never");
}

#[must_use]
pub fn build_with_build_wrap() -> Command {
    let build_wrap = env!("CARGO_BIN_EXE_build-wrap");

    let mut command = cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{build_wrap}'"),
    ]);

    command
}

#[must_use]
pub fn build_with_default_linker() -> Command {
    let mut command = cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{DEFAULT_LD}'"),
    ]);

    command
}

pub fn temp_package<'a, 'b>(
    build_script_path: Option<impl AsRef<Path>>,
    dependencies: impl IntoIterator<Item = (&'a str, &'b str)>,
) -> Result<TempDir> {
    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    if let Some(build_script_path) = build_script_path {
        copy(build_script_path, tempdir.path().join("build.rs"))?;
    }
    create_dir(tempdir.path().join("src"))?;
    write(tempdir.path().join("src/lib.rs"), "")?;

    let mut iter = dependencies.into_iter().peekable();

    if iter.peek().is_some() {
        let mut file = OpenOptions::new()
            .append(true)
            .open(tempdir.path().join("Cargo.toml"))?;
        writeln!(file, "\n[dependencies]")?;
        for (name, version) in iter {
            writeln!(file, r#"{name} = "{version}""#)?;
        }
    }

    Ok(tempdir)
}

const CARGO_TOML: &str = r#"
[package]
name = "temp-package"
version = "0.1.0"
edition = "2021"
publish = false

[build-dependencies]
libc = { version = "0.2", optional = true }
rustc_version = { version = "0.4", optional = true }
"#;

static METADATA: Lazy<Metadata> = Lazy::new(|| MetadataCommand::new().no_deps().exec().unwrap());

/// Creates a temporary directory in `build-wrap`'s target directory.
///
/// Useful if you want to verify that writing outside of the temporary directory is forbidden, but
/// `/tmp` is writeable, for example.
pub fn tempdir() -> Result<TempDir> {
    tempdir_in(&METADATA.target_directory).map_err(Into::into)
}
