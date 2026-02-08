# Upstream sync

Upstream (source of truth):
- https://github.com/anthropics/claude-agent-sdk-python

## Baseline
- Baseline is not explicitly tagged in this repo.
- Based on repo start date (2026-01-02) and upstream release dates, the likely initial baseline was around **v0.1.17**.

## Last synced
- Upstream release: v0.1.33 (2026-02-08)
- Upstream HEAD commit (at time of sync): ea81d412923c3e4d6b94ba770ea452cdfba3f51a

### Changes ported in this sync (v0.1.27 – v0.1.33)
- **v0.1.28**: Fix error field parsing (read from top-level, not nested message object)
- **v0.1.29**: New hook events (Notification, SubagentStart, PermissionRequest), new fields (tool_use_id, subagent fields, additional_context, updated_mcp_tool_output)
- **v0.1.31**: Always-streaming transport (remove --print mode), agents sent via initialize request, MCP tool annotations
- v0.1.27, v0.1.30, v0.1.32, v0.1.33: CLI-only version bumps (no SDK changes)

Notes:
- This file is maintained by the automated maintenance workflow.
