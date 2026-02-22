# Upstream sync

Upstream (source of truth):
- https://github.com/anthropics/claude-agent-sdk-python

## Baseline
- Baseline is not explicitly tagged in this repo.
- Based on repo start date (2026-01-02) and upstream release dates, the likely initial baseline was around **v0.1.17**.

## Last synced
- Upstream release: v0.1.39 (2026-02-22)
- Upstream HEAD commit (at time of sync): 146e3d6

### Changes ported in this sync (v0.1.34 – v0.1.39)
- **v0.1.36**: ThinkingConfig types (Adaptive/Enabled/Disabled), `thinking` field on ClaudeAgentOptions (supersedes `max_thinking_tokens`), `effort` option (low/medium/high/max)
- **v0.1.39**: Handle unknown message/content block types gracefully (return None instead of erroring)
- v0.1.34, v0.1.35, v0.1.37, v0.1.38: CLI-only version bumps (no SDK changes)

Notes:
- This file is maintained by the automated maintenance workflow.
