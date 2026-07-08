# Frontmatter in Prompt Files

`@file` loading (`eval_fp`) reads raw file content and interpolates variables only — it does not parse YAML frontmatter. So `model: sonnet` in a prompt `.md` file's frontmatter is just text in the prompt, not a runtime config.

This differs from directory-mode `.md` files where `tree.rs` parses frontmatter for `agent`, `model`, `on_fail`, `compact`.

If `@file` loading also parsed frontmatter and applied it (e.g., setting the model for the `do` call), prompt files could self-declare their agent requirements without duplicating `using` in the script.
