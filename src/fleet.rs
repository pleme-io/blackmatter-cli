//! Fleet data model — what the aggregator flake exposes that this CLI
//! reads. Everything here is typed; there are no stringly-typed paths
//! escaping to shell.

use crate::nix;
use anyhow::Result;

/// Enumerate the check attribute names under `checks.<system>` on the
/// given flake. Returns them sorted for deterministic output.
pub fn list_checks(flake: &str, system: &str) -> Result<Vec<String>> {
    let attr = format!("checks.{system}");
    // Use `--apply builtins.attrNames` so we get an array of names rather
    // than the full derivation tree.
    let value = nix::eval_json_apply(flake, &attr, "cs: builtins.attrNames cs")?;
    let mut names: Vec<String> = serde_json::from_value(value)?;
    names.sort();
    Ok(names)
}
