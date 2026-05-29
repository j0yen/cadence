# cadence — changelog

## v0.2.0

Adds the `pulse` subcommand: a one-command per-tier overdue readout that
consumes the substrate records written by v0.1.0 tools.

### New: `cadence pulse`

- `cadence pulse` — human-readable table showing tier, last-produced
  timestamp, expected cadence, and status (`ok` / `overdue: Xd` /
  `overdue: never`). Overdue rows print in red when stdout is a TTY.
- `cadence pulse --json` — machine-readable JSON array (or single object
  with `--tier`). Each element includes `tier`, `last_produced_at`,
  `age_days`, `cadence_days`, `overdue_after_days`,
  `overdue_delta_days`, and `status`.
- `cadence pulse --hook` — terse, one-line-per-overdue-tier output to
  stderr, suitable for `SessionStart` hook injection. Nothing on stdout.
- `cadence pulse --tier <daily|weekly|monthly|quarterly|annual>` — single
  tier view.
- `cadence pulse --quiet` — no output; communicate only via exit code.

Exit codes: `0` = all current; `1..=5` = number of overdue tiers;
`127` = substrate not initialized.

Default overdue thresholds (configurable via `tiers.<name>` in
`manifest.json`):

| Tier      | cadence | overdue_after |
|-----------|---------|---------------|
| daily     | 1d      | 2d            |
| weekly    | 7d      | 14d           |
| monthly   | 30d     | 60d           |
| quarterly | 92d     | 184d          |
| annual    | 365d    | 730d          |

### New: `scripts/cadence-pulse-hook.sh`

Shell helper that wraps `cadence pulse --hook` for easy `SessionStart`
integration. See the script's header for the one-line install snippet.

---

## v0.1.0

Foundational substrate. Single Rust binary `cadence` with four primary
subcommands (plus `register` for setup):

- `record <tier>` — append a new record under `$CADENCE_HOME/<tier>/<period>/`.
- `list <tier>` — enumerate records (filters: `--since`, `--period`,
  `--produced-by`; `--json` for machine output).
- `latest <tier>` — return the newest record for a tier (optionally
  filtered by `--produced-by`).
- `where` — print `$CADENCE_HOME`, the manifest path, and per-tier
  record counts.
- `register <name> --tier <t>` (setup helper) — declares a tool's
  intent to record under a tier, persisted in `manifest.json`.

Directory schema under `$CADENCE_HOME` (default `~/.claude/cadence/`):

```
$CADENCE_HOME/
├── manifest.json
├── daily/<YYYY-MM-DD>/<ulid>.json
├── weekly/<YYYY-Www>/<ulid>.json
├── monthly/<YYYY-MM>/<ulid>.json
├── quarterly/<YYYY-Q[1-4]>/<ulid>.json
└── annual/<YYYY>/<ulid>.json
```

Period derivation uses the local timezone. Weekly periods follow ISO
8601 (`Www`). Records are append-only: every `cadence record`
invocation writes a fresh ULID-named file; nothing is mutated or
deleted by this binary.
