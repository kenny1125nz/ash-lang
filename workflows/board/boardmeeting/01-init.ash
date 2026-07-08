#!opencode:1.0

DATE = $(date +%Y-%m-%d)
OUT = "board-output/${DATE}"
exec mkdir -p "${OUT}"

MINUTES = "${OUT}/meeting-minutes.md"

PROFILES = ["marcus-thorne", "priya-nair", "sarah-chen", "elena-vasquez", "james-okonkwo"]
NON_CEO = ["priya-nair", "sarah-chen", "elena-vasquez", "james-okonkwo"]
DIR = "directors"
INPUTS = "inputs/${DATE}.md"

exec "printf '%s\n' '# Ash Board Meeting Minutes — ${DATE}' '' '---' '' '## Attendance' '' > ${MINUTES}"
for SLUG in PROFILES {
  exec "echo '- ${SLUG}' >> ${MINUTES}"
}
exec "printf '\n%s\n' '---' '' 'Meeting called to order.' '' >> ${MINUTES}"

print "=== Ash Board Meeting ==="
print "Date: ${DATE}"
print "Minutes: ${MINUTES}"
print ""
print "Meeting called to order."
