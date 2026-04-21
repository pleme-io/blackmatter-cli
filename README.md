# blackmatter-cli

Typed Rust CLI for the [blackmatter](https://github.com/pleme-io/blackmatter)
fleet. Replaces the shell-scripted `nix run` apps that previously lived on
the aggregator with a properly typed binary — no bash, no string munging,
no shell escaping.

## Commands

```bash
# Pretty-print the per-component audit report (pure Nix data).
blackmatter-cli report

# List every rolled-up check the aggregator exposes on this system.
blackmatter-cli checks

# Build every rolled-up check; fail on first failure.
blackmatter-cli check
blackmatter-cli check --keep-going

# Build only checks for a single component.
blackmatter-cli check-component opencode
blackmatter-cli check-component ghostty --keep-going

# Target a different flake (default: github:pleme-io/blackmatter).
blackmatter-cli --flake path:/Users/you/code/github/pleme-io/blackmatter report

# Override the target system.
blackmatter-cli --system x86_64-linux checks
```

## Installation

Via Nix:

```bash
nix run github:pleme-io/blackmatter-cli -- report
nix profile install github:pleme-io/blackmatter-cli
```

Via home-manager (see `blackmatter-cli.homeManagerModules.default` once
the HM module is published).

## How it talks to Nix

Every Nix interaction is a typed subprocess call (`std::process::Command`)
with arguments built from typed pieces. There is no shell interpolation,
no bash, and no `writeShellScript` anywhere in the tree. The data flows:

```
github:pleme-io/blackmatter
  ├── fleet-report         (pure Nix string → `nix eval --raw`)
  └── checks.<sys>.<name>  (derivations    → `nix build --no-link`)
         │
         └─→ blackmatter-cli parses, iterates, reports
```

## Development

```bash
nix develop
cargo fmt
cargo clippy
cargo test
# or:
nix run .#check-all
```

Regenerate `Cargo.nix` after dependency changes:

```bash
nix run .#regenerate-cargo-nix
```

## Release

```bash
nix run .#bump -- patch
nix run .#release
```

## License

MIT
