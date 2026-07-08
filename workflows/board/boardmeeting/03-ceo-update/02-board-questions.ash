#!opencode:1.0

print "=== Board Questions for the CEO ==="

for SLUG in NON_CEO {
  print ""
  print "--- ${SLUG} asks questions ---"

  do "Read the profile file named directors/${SLUG}.md. You ARE this director — adopt their role, background, positions, and personality. Also read the company vision (vision.md) and mission (mision.md).

**Read the board inputs at ${INPUTS}.** Ground your questions in the actual data.

Read the CEO's update in ${MINUTES} under '2. CEO Business Update'.

Based on your profile's area of expertise, ask your hardest questions: investor probes burn/runway, community probes contributor health, enterprise probes compliance/procurement, GTM probes conversion/funnel.

**CRITICAL: Ask exactly 3 questions. One sentence each. No preamble, no commentary, no paragraphs.** Append to ${MINUTES} under: 'Board Questions — [your name]'. Write in first person. Be direct." with opencode
}

print ""
print "--- CEO responds ---"
