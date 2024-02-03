use anyhow::{anyhow, Result};
use std::{
    fs::{create_dir, write},
    path::Path,
};
use tempfile::{tempdir, TempDir};

pub fn package(build_script_path: &Path) -> Result<TempDir> {
    let build_script_path_as_str = build_script_path
        .to_str()
        .ok_or_else(|| anyhow!("build script path is not valid unicode"))?;

    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    create_dir(tempdir.path().join("src"))?;
    write(
        tempdir.path().join("src/main.rs"),
        main_rs(build_script_path_as_str),
    )?;

    Ok(tempdir)
}

const CARGO_TOML: &str = r#"
[package]
name = "build_script_wrapper"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
anyhow = "1.0"
tempfile = "3.9"
"#;

/// A wrapper build script's src/main.rs consists of the following:
///
/// - the contents of util.rs (included verbatim)
/// - the original build script as a byte slice (`BYTES`)
/// - a `main` function
///
/// See [`package`].
fn main_rs(build_script_path_as_str: &str) -> Vec<u8> {
    [
        UTIL_RS,
        &format!(
            r#"
const BYTES: &[u8] = include_bytes!("{build_script_path_as_str}");

fn main() -> Result<()> {{
    unpack_and_exec(BYTES)
}}
"#,
        )
        .as_bytes(),
    ]
    .concat()
}

const UTIL_RS: &[u8] = include_bytes!("util.rs");