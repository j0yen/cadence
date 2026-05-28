# cadence — the shared time-pyramid record store

This laptop has five reflective-artifact tools at five time horizons —
`daily-receipt`, `confidant`, `letters-we-never-sent`,
`conversations-zine`, `memory-reliquary` — and none of them compose.
The blocker is that no shared record store exists where "I produced a
daily artifact today" can be looked up by "what daily records do I have
for this week?". `cadence` is that store: a small Rust CLI with
`record`, `list`, `latest`, `register`, and `where` subcommands over a
`~/.claude/cadence/` directory layout. Foundational only — no tier
wiring happens here; the per-tool binds come in follow-on PRDs.

## Commands

```
cadence register <name> --tier <tier>   # declare a tool's intent to record
cadence record <tier> --produced-by <tool> --path <p> [--summary ...] [--sources ...]
cadence list <tier> [--since 7d] [--period ...] [--produced-by ...] [--json]
cadence latest <tier> [--produced-by <tool>] [--json]
cadence where                           # print $CADENCE_HOME + per-tier counts
```

Records are **append-only**: two records on the same day by the same
tool both persist; `latest` returns the newer one. The substrate root
is `~/.claude/cadence/` (override via `CADENCE_HOME`); records land
under `<tier>/<period>/<ulid>.json` where `<period>` is the calendar
bucket for the tier (e.g. `daily/2026-05-28/`, `weekly/2026-W22/`).

## Install

```sh
cargo install --path .
# installs the `cadence` binary to ~/.cargo/bin (or ~/.local/bin via
# the build skill's install -Dm755 step)
cadence --version   # 0.1.0
cadence where       # creates ~/.claude/cadence/ on first run
```

## License

Dual-licensed under MIT OR Apache-2.0, © Joe Yen.
