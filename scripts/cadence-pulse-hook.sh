#!/usr/bin/env bash
# cadence-pulse-hook.sh — SessionStart hook integration for `cadence pulse`.
#
# Calls `cadence pulse --hook` and, if any tiers are overdue, prints a brief
# nudge formatted for SessionStart hook output.
#
# Install (add to ~/.claude/settings.json hooks.SessionStart):
#   {
#     "hooks": {
#       "SessionStart": [
#         {
#           "matcher": "",
#           "hooks": [
#             { "type": "command", "command": "/path/to/cadence-pulse-hook.sh" }
#           ]
#         }
#       ]
#     }
#   }
#
# Or add this line to your existing SessionStart script:
#   /path/to/scripts/cadence-pulse-hook.sh
#
# The script exits 0 unconditionally so a non-zero cadence exit code does not
# abort the SessionStart hook chain. Overdue tier messages go to stderr.

set -euo pipefail

CADENCE_BIN="${CADENCE_BIN:-cadence}"

if ! command -v "$CADENCE_BIN" &>/dev/null; then
    # Quietly skip if cadence is not installed yet.
    exit 0
fi

# Run pulse --hook; capture exit code without aborting.
"$CADENCE_BIN" pulse --hook 2>&1 || OVERDUE=$?

OVERDUE="${OVERDUE:-0}"

if [[ "$OVERDUE" -eq 127 ]]; then
    # Substrate not initialized — not an error worth surfacing on every session.
    exit 0
fi

if [[ "$OVERDUE" -gt 0 ]]; then
    echo "  [cadence] ${OVERDUE} tier(s) overdue — run 'cadence pulse' for details" >&2
fi

exit 0
