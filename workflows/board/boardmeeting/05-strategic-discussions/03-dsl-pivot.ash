#!opencode:1.0

TOPIC = "DSL Engineering Investment vs Markdown Engine Pivot"
DISC = "${OUT}/discussion-dsl-pivot.md"

exec "printf '%s\n' '# Discussion: ${TOPIC}' '' > ${DISC}"

print "=== Functional Deep Dive: ${TOPIC} ==="

RELEVANT = ["marcus-thorne", "priya-nair", "elena-vasquez", "sarah-chen"]

for SLUG in RELEVANT {
  print ""
  print "--- ${SLUG} speaks on ${TOPIC} ---"

  do "Read the profile file named directors/${SLUG}.md. You ARE this director — adopt their role, background, board positions, and perspective. Also read the company vision (vision.md), mission (mision.md), and the board inputs at ${INPUTS}.

Read the meeting minutes at ${MINUTES} for context. Read the discussion so far at ${DISC} to see prior positions.

**Append your full position to ${DISC}** under '## [your name]'. Argue your position on: **Continue DSL investment or pivot to markdown engine?** Your profile has your stance — advocate it. Address:

- Your recommendation
- Why this best serves the company's trajectory
- Resource implications (what should we stop or start doing)
- What role the DSL should play if not the primary surface

The CEO has emotional attachment to the parser. Be honest but constructive. Write in your voice. No token limit." with opencode
}

print ""
print "--- Chair summarizes into minutes ---"

do "Read the full discussion at ${DISC}. Also read vision (vision.md) and mission (mision.md).

You are the board chair (read directors/sarah-chen.md for context).

**Append to ${MINUTES}** under section '5. Functional Deep Dive — DSL vs Markdown Engine' a summary:

1. Positions: one sentence per director
2. Agreement
3. Tension (engineering attachment vs business reality)
4. Resolution: one sentence beginning 'RESOLVED THAT...' (include conditions if any)
5. Mission check: which pillar does this advance?
6. Dissent

**6 bullets total. One sentence each.** Full debate: ${DISC}." with opencode
