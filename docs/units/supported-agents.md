---
{"id": "supported-agents", "title": "Supported Agents"}
---

## Supported Agents

| Agent | Description |
|-------|-------------|
| `echo` | Built-in passthrough for testing |
| `opencode` | OpenCode CLI agent |
| `claude-code` | Anthropic Claude Code |
| `aider` | Aider AI pair programming |

### Auto-discovery

Agents are auto-discovered on your PATH. Run to refresh:

```bash
ash discover
```

### Custom agents

Add custom CLI-based agents in `ash.yml`:

```yaml
agents:
  my-tool:
    type: local-cli
    cmd: my-tool
    message_flag: "--prompt"
    yes_flag: "--yes"
```
