# cadence — changelog

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
