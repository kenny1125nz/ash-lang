Read tmp/task-definition_${IDX}.md and give it a rigorous critical review based on original requirement ${INPUT_FILE} . Do not rubber-stamp — challenge assumptions, identify weak spots, and be honest about quality. A surface-level assessment that skims through the criteria is worse than none.

Guide your review by what makes a good task definition:

- **Scope discipline** — does it solve exactly what was asked, or does it invent extra features and edge cases? Unrequested additions are a liability, not a virtue.
- **Design quality** — does the Intended Solution show genuine design thinking with trade-offs and rationale, or just a description of what to build? Good designs explain why, not just what.
- **Actionability** — can a developer pick this up and implement it without guessing? Vague or ambiguous sections undermine execution.
- **Context awareness** — does it reference the actual codebase, existing patterns, files, and conventions? Generic advice is useless.
- **Implementation Hints** — are they specific about what NOT to touch? This is often more valuable than what to do.

Create tmp/assessment-report.md with these sections:

**Findings** — numbered list of issues. Each finding must include a severity label (critical, major, minor) and reference specific parts of the document. Focus on what genuinely undermines quality rather than nitpicking style.

**Strengths** — what works well and should be preserved.

Then create tmp/score.<SCORE> — an empty file whose name contains the overall score (0-100), e.g. tmp/score.85. Be honest: 85 means genuinely good, not just passing. Reserve 90+ for exceptional work.