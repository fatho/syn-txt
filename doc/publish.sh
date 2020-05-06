#!/usr/bin/env bash

set -euo pipefail

# Build the docs
docout=$(nix-build --no-out-link -A syn-txt-doc)

# Publish the generated documentation on the gh-pages branch

worktree=$(mktemp -d)
git worktree add "$worktree" gh-pages
echo Temporary checkout at $worktree
pushd "$worktree"

# Make sure it's up to date
git pull

# Remove everything
git rm -r .

# Copy documentation output
cp -r $docout/html/* "$worktree"
chmod -R +w "$worktree"

# Ensure GitHub doesn't try to do anything fancy with our stuff
touch "$worktree/.nojekyll"

# Commit changes
git add .
git commit

# Push
git push -u origin gh-pages

echo "done"

git worktree remove "$worktree"