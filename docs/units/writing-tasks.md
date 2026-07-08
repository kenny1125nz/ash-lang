---
{"id": "writing-tasks", "title": "Writing Tasks"}
---

## Writing Tasks

Each `.md` file is a standalone prompt sent to the agent. Optional YAML frontmatter sets per-task config (agent, model, etc.). The filename sets the order — Ash sorts alphanumerically. Subdirectories group related tasks.

**tasks/1-plan/01-requirements.md:**

```markdown
---
agent: opencode
---

Write a requirements document for a login system that supports:

- Email/password authentication
- OAuth with Google and GitHub
- Session management with JWT
- Rate limiting on failed attempts
```

**tasks/2-implement/01-auth.md:**

```markdown
---
agent: claude-code
---

Implement the login API endpoint. Cover:

- POST /auth/login — validates email/password, returns JWT
- POST /auth/register — creates user, sends verification email
- POST /auth/refresh — refreshes expired tokens

Use the requirements from tasks/1-plan/01-requirements.md.
```

### Frontmatter reference

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `agent` | agent name | — | Override agent for this task |
| `model` | model name | — | Override model for this task |
| `on_fail` | `stop`, `continue` | `stop` | Behavior when the task fails |
| `compact` | directive | — | Context window strategy for this task |
