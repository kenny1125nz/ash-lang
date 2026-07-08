#!opencode:1.0

TOPIC = "Ash Cloud Monetization & Pricing Model"
DISC = "${OUT}/discussion-monetization.md"

exec "printf '%s\n' '# Discussion: ${TOPIC}' '' > ${DISC}"

print "=== Strategic Discussion: ${TOPIC} ==="

for SLUG in PROFILES {
  print ""
  print "--- ${SLUG} speaks on ${TOPIC} ---"

  do "Read the profile file named directors/${SLUG}.md. You ARE this director — adopt their role, background, board positions, and perspective. Also read the company vision (vision.md), mission (mision.md), and the board inputs at ${INPUTS}.

Read the meeting minutes at ${MINUTES} for meeting context. Read the discussion so far at ${DISC} to see prior positions.

**Append your full position to ${DISC}** under '## [your name]'. Argue your position on: **What pricing model for Ash Cloud, and how do we reach enterprise revenue?** Your profile has your stance — advocate it. Address:

- Your recommended pricing model and why
- Specific free tier threshold
- Path from developer adoption to enterprise revenue
- Key risk or condition for your recommendation to work

Respond to prior positions if you directly disagree. Be concrete. Use the data from the inputs. Write in your voice. No token limit — this is the full debate record." with opencode
}

print ""
print "--- Chair summarizes into minutes ---"

do "Read the full discussion at ${DISC}. Also read vision (vision.md) and mission (mision.md).

You are the board chair (read directors/marcus-thorne.md for context).

**Append to ${MINUTES}** under section '3. Strategic Discussion — Monetization' a summary:

1. Positions: one sentence per director capturing their core recommendation
2. Agreement: what everyone aligned on
3. Trade-off: the key disagreement or tension
4. Resolution: one sentence beginning 'RESOLVED THAT...'
5. Mission check: which pillar does this advance?
6. Dissent: note if any, else 'None'

**6 bullets total. One sentence each.** The full debate is archived at ${DISC}." with opencode
