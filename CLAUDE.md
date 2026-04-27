# blackmatter-cli — Claude Orientation

> **★★★ CSE / Knowable Construction.** This repo operates under
> **Constructive Substrate Engineering** — canonical specification at
> [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md).
> The Compounding Directive (operational rules: solve once, load-bearing
> fixes only, idiom-first, models stay current, direction beats velocity)
> is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before
> non-trivial changes. Canonical NO-SHELL replacement for the
> `pkgs.writeShellScript` apps on the blackmatter aggregator — every fleet
> orchestration verb (fleet-check, ci-fleet, ci-component) is a typed
> Rust subcommand invoking `nix` with argv built from typed pieces.

One-sentence purpose: typed Rust CLI that replaces shell-scripted apps
on the `blackmatter` aggregator. Reads `fleet-report` + rolled-up
`checks.<sys>.*` via typed `nix` subprocess calls and orchestrates them.

## Classification

- **Archetype:** `RustTool`
- **Flake shape:** `substrate/lib/rust-tool-release-flake.nix`
- **Binary:** `blackmatter-cli`
- **Systems:** all 4 (aarch64-darwin, x86_64-darwin, x86_64-linux, aarch64-linux)

## Architectural law (why this repo exists)

Pleme-io's primary stack is **Rust + tatara-lisp + Nix + YAML** (Pangea-Ruby
for infrastructure). Shell scripts are forbidden beyond 3-line inline glue.
An earlier iteration wrote `pkgs.writeShellScript` apps on the blackmatter
aggregator for `fleet-check` / `ci-fleet` / `ci-component`. That was the
wrong pattern. This repo is the correct replacement: a typed Rust binary
that invokes `nix` as a subprocess with argv built from typed pieces — no
shell interpolation, no string munging.

## Where to look

| Intent | File |
|--------|------|
| CLI entry + subcommands (clap) | `src/main.rs` |
| Typed nix subprocess wrapper  | `src/nix.rs` |
| Fleet data model + enumeration | `src/fleet.rs` |
| Flake (substrate builder)      | `flake.nix` |
| Typescape manifest             | `.typescape.yaml` |

## What NOT to do

- **Don't add a shell dependency.** The whole point is that this replaces
  shell. If you need a new operation, extend `src/nix.rs` with a new typed
  wrapper around the relevant `nix` subcommand — never `writeShellScript`
  or `bash -c`.
- **Don't parse nix store paths by string slicing.** Use `nix eval --json`
  with serde types; let the compiler enforce shape.
- **Don't inline shell-style argv assembly.** Each `Command::new("nix")`
  call lists its args as typed pieces; interpolation only happens inside
  `&format!(...)` for attribute paths, which are validated upstream.

## Design surface

- `Cli::system` defaults to the host's `std::env::consts::ARCH`/`OS`
  mapped to a Nix system string (`aarch64-darwin`, `x86_64-linux`, …).
- `Cli::flake` defaults to `github:pleme-io/blackmatter` but accepts any
  valid flake URI (`path:…`, `git+ssh:…`, etc.).
- `Command::Check` walks the checks list in sorted order; `--keep-going`
  continues past failures, otherwise exits on first.
- Future: `Command::Publish` that uploads a signed fleet attestation
  (typescape Merkle root) to a registered sink.

## Future extensions

- Replace `list_checks` 's inline lambda string with a typed `--apply-file`
  pattern once substrate exposes a helper.
- Declarative fleet spec via `shikumi` YAML (`~/.config/blackmatter/cli.yaml`).
- `#[derive(TataraDomain)]` on a `FleetSpec` Rust struct for Lisp authoring
  via `(defblackmatterfleet …)`.
