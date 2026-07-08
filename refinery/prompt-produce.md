Analyze the requirement from the source files and design a solid solution. Turn your analysis into a task definition written to tmp/task-definition_${IDX}.md.

Apply these principles:
- **Scope Discipline** — solve what the requirement asks for, nothing more. Avoid inventing extra features, edge cases, or requirements that weren't requested. Every unrequested addition is a maintenance burden.
- **Surface, Don't Hide** — prefer clear errors over defensive fallbacks. Avoid adding fallback logic that masks problems. Ensure issues surface immediately so they get fixed properly.
- **Think Then Build** — thoroughly analyze the codebase and the situation before designing. Avoid patching changes without understanding the bigger picture. Ensure the design is sound from the start by investing in understanding upfront.

Structure the file with these sections:

- **Background** — the problem, current state, why it matters
- **Intended Solution** — the design with concrete decisions, trade-offs, and rationale. .
- **Acceptance Criteria** — numbered, testable, specific conditions that define done
- **Implementation Hints** — project context: relevant modules, file paths, existing patterns, what NOT to touch. Do not prescribe struct names or step-by-step instructions.

Use these source files:
- Start with ${INPUT_FILE} for the original requirement.
- Check tmp/task-definition_${PREV_IDX}.md as the previous version, if available, to build upon, with findings covered in tmp/assessment-report.md if available to improve upon.