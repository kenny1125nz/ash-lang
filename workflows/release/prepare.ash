last = $(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || git rev-list --max-parents=0 HEAD)
log = $(git log ${last}..HEAD --oneline)
diff = $(git diff --stat ${last}..HEAD)

exec git tag -a "${tag}" -m "Release ${tag}"
exec git push origin "${tag}"

exec mkdir -p ./tmp
notes = "./tmp/release-notes-${tag}.md"
