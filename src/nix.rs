//! Thin typed wrapper around the `nix` CLI. All invocations construct
//! their argv from typed pieces — no shell, no string interpolation into
//! a shell context.
//!
//! The `nix_base!` macro factors the duplicated `Command` construction
//! every typed wrapper would otherwise repeat. Each wrapper adds only
//! the operation-specific flags.

use anyhow::{Context, Result, bail};
use std::process::Output;

/// Construct a `Command` pre-populated with `nix <subcommand>
/// --accept-flake-config <flake>#<attr>`. Caller chains operation-specific
/// flags (e.g. `.arg("--raw")`, `.arg("--no-link")`) before `.output()`
/// or `.status()`.
///
/// Extracted because every typed wrapper in this module would otherwise
/// repeat the same three arg-push calls — per the prime directive
/// (macros everywhere, duplication is a bug).
#[macro_export]
macro_rules! nix_base {
    ($subcommand:literal, $flake:expr, $attr:expr) => {{
        let mut cmd = ::std::process::Command::new("nix");
        cmd.arg($subcommand)
            .arg("--accept-flake-config")
            .arg(format!("{}#{}", $flake, $attr));
        cmd
    }};
}

/// Helper: fail loudly if a `nix` subprocess exited non-zero, capturing
/// stderr for the error message.
fn ensure_success(out: &Output, what: &str) -> Result<()> {
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("{what} failed: {stderr}");
    }
    Ok(())
}

/// `nix eval --accept-flake-config <flake>#<attr> --raw` → stdout as `String`.
pub fn eval_raw(flake: &str, attr: &str) -> Result<String> {
    let out = nix_base!("eval", flake, attr)
        .arg("--raw")
        .output()
        .with_context(|| format!("spawning `nix eval --raw {flake}#{attr}`"))?;
    ensure_success(&out, &format!("nix eval --raw {flake}#{attr}"))?;
    String::from_utf8(out.stdout).context("nix eval output was not utf-8")
}

/// `nix eval --accept-flake-config <flake>#<attr> --json` → parsed JSON value.
#[allow(dead_code)]
pub fn eval_json(flake: &str, attr: &str) -> Result<serde_json::Value> {
    let out = nix_base!("eval", flake, attr)
        .arg("--json")
        .output()
        .with_context(|| format!("spawning `nix eval --json {flake}#{attr}`"))?;
    ensure_success(&out, &format!("nix eval --json {flake}#{attr}"))?;
    serde_json::from_slice(&out.stdout).context("parsing nix eval --json output")
}

/// `nix eval --accept-flake-config <flake>#<attr> --json --apply <lambda>` →
/// parsed JSON value. Use this to transform an attrset (e.g. a checks set)
/// into a serialisable shape like `builtins.attrNames` before decoding.
pub fn eval_json_apply(flake: &str, attr: &str, apply: &str) -> Result<serde_json::Value> {
    let out = nix_base!("eval", flake, attr)
        .args(["--json", "--apply", apply])
        .output()
        .with_context(|| format!("spawning `nix eval --apply {flake}#{attr}`"))?;
    ensure_success(&out, &format!("nix eval --apply {flake}#{attr}"))?;
    serde_json::from_slice(&out.stdout).context("parsing nix eval --apply output")
}

/// `nix build --no-link --accept-flake-config <flake>#<attr>`; returns
/// `Ok(())` iff the derivation builds (or is already built in the store).
pub fn build(flake: &str, attr: &str) -> Result<()> {
    let status = nix_base!("build", flake, attr)
        .arg("--no-link")
        .status()
        .with_context(|| format!("spawning `nix build {flake}#{attr}`"))?;
    if !status.success() {
        bail!("nix build {flake}#{attr} failed (exit {})", status);
    }
    Ok(())
}
