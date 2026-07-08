#!opencode:1.0

TOPIC = "Open-Source Licensing & Moat Defense"
DISC = "${OUT}/discussion-licensing.md"

exec "printf '%s\n' '# Discussion: ${TOPIC}' '' > ${DISC}"

print "=== Strategic Discussion: ${TOPIC} ==="

for SLUG in PROFILES {
  print ""
  print "--- ${SLUG} speaks on ${TOPIC} ---"

  do "Read the profile file named directors/${SLUG}.md. You ARE this director — adopt their role, background, board positions, and perspective. Also read the company vision (vision.md), mission (mision.md), and the board inputs at ${INPUTS}.

Read the meeting minutes at ${MINUTES} for context. Read the discussion so far at ${DISC} to see prior positions.

**Append your full position to ${DISC}** under '## [your name]'. Argue your position on: **Should Ash stay MIT or adopt a defensive license?** Your profile has your stance — advocate it. Address:

- Your recommendation (MIT or which alternative)
- Why this protects Ash better than the other path
- The biggest risk of your approach
- What else must be true for your choice to work

If your profile has direct experience (BSL at HashiCorp, enterprise procurement impact, GTM implications), bring it to bear. Write in your voice. No token limit." with opencode
}

print ""
print "--- Chair summarizes into minutes ---"

do "Read the full discussion at ${DISC}. Also read vision (vision.md) and mission (mision.md).

You are the board chair (read directors/priya-nair.md for context).

**Append to ${MINUTES}** under section '4. Strategic Discussion — Licensing' a summary:

1. Positions: one sentence per director
2. Agreement
3. Trade-off
4. Resolution: one sentence beginning 'RESOLVED THAT...' (if MIT, add what else must hold)
5. Mission check: which pillar does this advance?
6. Dissent

**6 bullets total. One sentence each.** Full debate: ${DISC}." with opencode
