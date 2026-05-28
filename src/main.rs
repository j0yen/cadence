//! `cadence` CLI entry point.

#![cfg_attr(not(test), forbid(unsafe_code))]
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::BTreeMap;
use std::process::ExitCode;

use anyhow::{Context, Result};
use cadence::{
    latest, list, parse_since, record_store, register, resolve_home, where_stats,
    ListFilter, Tier,
};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

/// cadence — shared time-pyramid record store.
#[derive(Parser, Debug)]
#[command(name = "cadence", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Register a tool's intent to record under a tier.
    Register {
        /// Tool name (matches `produced-by` on `record`).
        name: String,
        /// Tier the tool records under.
        #[arg(long)]
        tier: Tier,
    },
    /// Append a new record under the appropriate period directory.
    Record {
        /// Tier.
        tier: Tier,
        /// Producing tool's name.
        #[arg(long = "produced-by")]
        produced_by: String,
        /// Filesystem path to the artifact.
        #[arg(long)]
        path: String,
        /// Optional human-readable summary.
        #[arg(long)]
        summary: Option<String>,
        /// Comma-separated cadence record IDs this record composes from.
        #[arg(long, value_delimiter = ',')]
        sources: Vec<String>,
        /// `key=value` metadata pairs (repeatable).
        #[arg(long, value_parser = parse_kv)]
        meta: Vec<(String, String)>,
        /// Override the produced-at timestamp (RFC 3339).
        #[arg(long = "produced-at")]
        produced_at: Option<DateTime<Utc>>,
        /// Override the derived period string.
        #[arg(long)]
        period: Option<String>,
    },
    /// Enumerate records under a tier.
    List {
        /// Tier.
        tier: Tier,
        /// Filter to records produced within this duration window (e.g. `7d`, `2w`).
        #[arg(long)]
        since: Option<String>,
        /// Filter to a specific period string (e.g. `2026-W21`).
        #[arg(long)]
        period: Option<String>,
        /// Filter to records produced by this tool.
        #[arg(long = "produced-by")]
        produced_by: Option<String>,
        /// Emit JSON array on stdout instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// Print the newest record for a tier.
    Latest {
        /// Tier.
        tier: Tier,
        /// Filter to a specific producing tool.
        #[arg(long = "produced-by")]
        produced_by: Option<String>,
        /// Emit JSON on stdout instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// Print `$CADENCE_HOME` paths and per-tier counts.
    Where {
        /// Emit JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
}

fn parse_kv(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got '{s}'"))?;
    Ok((k.to_string(), v.to_string()))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cadence: error: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let home = resolve_home().context("resolve CADENCE_HOME")?;
    match cli.cmd {
        Cmd::Register { name, tier } => {
            register(&home, &name, tier)?;
            println!("registered {name} → {tier}");
        }
        Cmd::Record {
            tier,
            produced_by,
            path,
            summary,
            sources,
            meta,
            produced_at,
            period,
        } => {
            let meta_map: BTreeMap<String, String> = meta.into_iter().collect();
            let rec = record_store(
                &home,
                tier,
                &produced_by,
                &path,
                produced_at,
                period.as_deref(),
                summary.as_deref(),
                sources,
                meta_map,
            )?;
            println!("{}", rec.id);
        }
        Cmd::List {
            tier,
            since,
            period,
            produced_by,
            json,
        } => {
            let mut filter = ListFilter::default();
            if let Some(s) = since {
                filter.since = Some(parse_since(&s)?);
            }
            filter.period = period;
            filter.produced_by = produced_by;
            let items = list(&home, tier, &filter)?;
            if json {
                let s = serde_json::to_string_pretty(&items).context("serialize list")?;
                println!("{s}");
            } else {
                for it in &items {
                    println!(
                        "{}  {}  {}  {}",
                        it.id,
                        it.produced_at.format("%Y-%m-%dT%H:%M:%SZ"),
                        it.produced_by,
                        it.path
                    );
                }
                if items.is_empty() {
                    println!("(no records)");
                }
            }
        }
        Cmd::Latest {
            tier,
            produced_by,
            json,
        } => {
            let filter = ListFilter {
                produced_by,
                ..Default::default()
            };
            match latest(&home, tier, &filter)? {
                Some(rec) if json => {
                    let s = serde_json::to_string_pretty(&rec).context("serialize latest")?;
                    println!("{s}");
                }
                Some(rec) => {
                    println!(
                        "{}  {}  {}  {}",
                        rec.id,
                        rec.produced_at.format("%Y-%m-%dT%H:%M:%SZ"),
                        rec.produced_by,
                        rec.path
                    );
                }
                None if json => {
                    println!("null");
                }
                None => {
                    println!("(no records)");
                }
            }
        }
        Cmd::Where { json } => {
            let s = where_stats(&home)?;
            if json {
                let v = serde_json::json!({
                    "home": s.home.display().to_string(),
                    "manifest": s.manifest_path.display().to_string(),
                    "counts": s.counts,
                });
                println!("{}", serde_json::to_string_pretty(&v).context("serialize where")?);
            } else {
                println!("home:     {}", s.home.display());
                println!("manifest: {}", s.manifest_path.display());
                for (k, v) in &s.counts {
                    println!("  {k:<10} {v}");
                }
            }
        }
    }
    Ok(())
}
