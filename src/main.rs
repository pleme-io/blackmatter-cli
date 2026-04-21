//! blackmatter-cli — typed orchestration over the blackmatter fleet.
//!
//! Reads the fleet's data surfaces (`fleet-report`, rolled-up
//! `checks.<sys>.*`) from the aggregator flake and drives them. Every
//! operation goes through `nix` as a subprocess with typed argument
//! construction — no shell scripts anywhere.

#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::io::Write;

mod fleet;
mod nix;

/// Typed CLI for the blackmatter fleet.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Flake reference for the blackmatter aggregator.
    #[arg(long, global = true, default_value = "github:pleme-io/blackmatter")]
    flake: String,

    /// Target system for per-system outputs (checks, apps).
    #[arg(long, global = true, default_value_t = current_system())]
    system: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print the fleet report (pure Nix data via `nix eval --raw .#fleet-report`).
    Report,

    /// List every rolled-up check available on the current system.
    Checks,

    /// Build every rolled-up check; report pass/fail per check.
    Check {
        /// Continue on first failure instead of exiting early.
        #[arg(long)]
        keep_going: bool,
    },

    /// Build only the checks for a single component.
    CheckComponent {
        /// Component short name (e.g. `anvil`, `opencode`). Matches
        /// checks prefixed with `<name>-`.
        name: String,

        /// Continue on first failure instead of exiting early.
        #[arg(long)]
        keep_going: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Report => cmd_report(&cli),
        Command::Checks => cmd_checks(&cli),
        Command::Check { keep_going } => cmd_check(&cli, None, *keep_going),
        Command::CheckComponent { name, keep_going } => {
            cmd_check(&cli, Some(name.as_str()), *keep_going)
        }
    }
}

fn cmd_report(cli: &Cli) -> Result<()> {
    let report = nix::eval_raw(&cli.flake, "fleet-report")
        .context("reading fleet-report from aggregator flake")?;
    print!("{report}");
    Ok(())
}

fn cmd_checks(cli: &Cli) -> Result<()> {
    let checks = fleet::list_checks(&cli.flake, &cli.system)
        .context("enumerating rolled-up checks")?;
    if checks.is_empty() {
        println!("(no checks registered for system {})", cli.system);
        return Ok(());
    }
    println!("{} check(s) on {}:", checks.len(), cli.system);
    for c in checks {
        println!("  • {c}");
    }
    Ok(())
}

fn cmd_check(cli: &Cli, filter: Option<&str>, keep_going: bool) -> Result<()> {
    let all = fleet::list_checks(&cli.flake, &cli.system)
        .context("enumerating rolled-up checks")?;
    let selected: Vec<String> = match filter {
        None => all,
        Some(name) => {
            let prefix = format!("{name}-");
            all.into_iter().filter(|c| c.starts_with(&prefix)).collect()
        }
    };

    if selected.is_empty() {
        if let Some(name) = filter {
            bail!("no checks found for component '{}' on {}", name, cli.system);
        }
        println!("(no checks to run)");
        return Ok(());
    }

    println!(
        "═══ blackmatter-cli check — system: {} — {} check(s) ═══",
        cli.system,
        selected.len()
    );

    let mut failures = Vec::new();
    for name in &selected {
        let attr = format!("checks.{}.{}", cli.system, name);
        print!("  • {name:<48} ");
        std::io::stdout().flush().ok();
        match nix::build(&cli.flake, &attr) {
            Ok(()) => println!("✓"),
            Err(e) => {
                println!("✗");
                failures.push((name.clone(), e));
                if !keep_going {
                    break;
                }
            }
        }
    }

    if failures.is_empty() {
        println!("\n✓ all {} check(s) passed", selected.len());
        Ok(())
    } else {
        println!("\n✗ {} failure(s):", failures.len());
        for (name, err) in &failures {
            println!("  {name}: {err}");
        }
        bail!("{} check(s) failed", failures.len())
    }
}

fn current_system() -> String {
    let arch = std::env::consts::ARCH;
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("{arch}-{os}")
}
