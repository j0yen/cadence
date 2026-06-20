# cadence

A record store for recurring artifacts across five time horizons — daily, weekly, monthly, quarterly, annual. A tool that produces something on a schedule records the fact here; anything can then ask "what daily records do I have this week?" or "which tiers are overdue?" without knowing where the artifacts live.

## Why it exists

Several reflective-artifact tools run at different horizons — a daily receipt, a weekly digest, and so on — and they don't compose, because there's no shared place to record "I produced a daily artifact today" that another step can query by period. cadence is that shared layer: each tool writes one record per artifact, and the time-pyramid (daily rolls up to weekly, weekly to monthly) becomes something you can query instead of reconstruct. It is the record store, not the scheduler — it tracks what was produced and when; it does not produce or trigger anything itself.

## Commands

```
cadence register <name> --tier <tier>   # declare a tool's intent to record under a tier
cadence record <tier> --produced-by <tool> --path <p> [--summary ...] [--sources ...]
cadence list <tier> [--since 7d] [--period ...] [--produced-by ...] [--json]
cadence latest <tier> [--produced-by <tool>] [--json]
cadence pulse [--tier <tier>] [--json] [--hook] [--quiet]   # which tiers are overdue
cadence where [--json]                   # print $CADENCE_HOME + per-tier counts
```

Tiers are `daily | weekly | monthly | quarterly | annual`.

Records are **append-only**: two records on the same day by the same tool both persist, and `latest` returns the newer one. The substrate root is `~/.claude/cadence/` (override via `CADENCE_HOME`); records land under `<tier>/<period>/<ulid>.json`, where `<period>` is the calendar bucket for the tier — `daily/2026-05-28/`, `weekly/2026-W22/`, `quarterly/2026-Q2/`, and so on.

`pulse` reports each tier's last-produced time against its expected cadence and exits with the number of overdue tiers (`127` if the substrate isn't initialized yet). `--hook` makes it terse and stderr-only for a SessionStart hook — `scripts/cadence-pulse-hook.sh` wires that up; `--quiet` drops all output and speaks only through the exit code.

## Install

```sh
cargo install --path .
# installs `cadence` to ~/.cargo/bin
cadence --version   # 0.2.0
cadence where       # creates ~/.claude/cadence/ on first run
```

## License

Dual-licensed under MIT OR Apache-2.0, © Joe Yen.
