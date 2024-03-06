use std::process::Command;

mod common;
pub use common::{exec, ToUtf8};

#[must_use]
pub fn cargo_build() -> Command {
    let mut command = Command::new("cargo");
    command.arg("build");

    // smoelius: Show build script (e.g., wrapper) output.
    // See: https://github.com/rust-lang/cargo/issues/985#issuecomment-258311111
    command.arg("-vv");

    // smoelius: Show linker output.
    // See: https://stackoverflow.com/a/71866183
    command.env("RUSTC_LOG", "rustc_codegen_ssa::back::link=info");

    command
}
