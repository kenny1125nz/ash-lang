Generate release notes for Ash version ${tag}.

Previous release: ${last}

Changes since last release:
```
${log}
```

File changes summary:
```
${diff}
```

Write a release notes blog post in markdown format to ${notes}. The content must start with a level-1 heading:

# Release ${tag}

Follow with a brief intro paragraph, then sections:

## What's New
- Feature descriptions (one sentence per bullet)

## Fixes
- Bug fix descriptions (one sentence per bullet)

## Improvements
- Performance, usability, or internal improvements worth noting

Be concise and user-facing. Do NOT include internal refactoring details unless they materially affect users. Write the complete file to ${notes}.
