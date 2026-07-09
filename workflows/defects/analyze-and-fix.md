Read ./tmp/defects.json — it contains open bug reports and feedback from the website.

For each defect:
1. Classify by priority: high (crash/broken page/error), medium (UX/visual), low (typo/minor)
2. Identify which files in ./src/ need changes
3. Write a summary to ./tmp/summary.md with defect id, category, priority, message, and affected files
4. Then FIX every defect in priority order. For each fix:
   - Edit the relevant source files
   - Verify the fix
   - Append the defect ID to ./tmp/fixed-ids.txt
5. After all fixes, run: npm run build
   Fix any build errors and retry until it passes.

If a defect cannot be fixed, write the ID and reason to ./tmp/unfixed.md
