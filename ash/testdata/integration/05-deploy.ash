# Deploy — commit and tag a release
_ = $(git add -A)
_ = $(git commit -m "Release v1.0.0")
_ = $(git tag v1.0.0)
print "Tagged release v1.0.0"