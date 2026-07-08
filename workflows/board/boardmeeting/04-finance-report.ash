#!opencode:1.0

print "=== Finance Report ==="

do "Read the board inputs at ${INPUTS} (especially section 5: Financials) and the company mission (mision.md). Generate a formal financial summary for the board. Combine the raw data from the inputs with narrative framing against the mission's three pillars. Include: executive summary, income statement with actuals vs prior quarter, burn analysis, runway projection, enterprise pipeline health, and key metric trends. Write to ${OUT}/finance-report.md." with opencode

print "Finance report saved to ${OUT}/finance-report.md"
