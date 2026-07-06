# Research: Echo driver session integration

Read `05-implementation.md` to understand the `session { }` block feature.

Echo is ash's built-in test/noop agent — no external docs to research. Update `05-implementation.md`'s `### Per-agent behavior` section:

- Add an `#### echo` subsection
- Confirmed no-op: `EchoDriver` silently ignores `ExecuteRequest.session`
