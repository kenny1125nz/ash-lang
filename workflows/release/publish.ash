if $? != 0 { print "failed to generate release notes"; exit 1 }

slug = $(echo "release-${tag}" | sed 's/[^a-zA-Z0-9]/-/g')
today = $(date +%Y-%m-%d)

exec node web-site/scripts/add-post.mjs "${notes}" --title "Release ${tag}" --slug "${slug}" --author "Ash Team" --date "${today}" --remote
