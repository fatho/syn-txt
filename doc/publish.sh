#!/usr/bin/env bash

set -euo pipefail

# Build the docs
docout=$(nix-build --no-out-link -A syntxt-doc)

# Build the web app
wasm=$(nix-build --no-out-link -A syntxt-wasm-release)

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

# Insert the demo
mkdir "$worktree/demo"
cp -r "$wasm"/* "$worktree/demo"

git add .

read -p "Are you sure? [y/n] Check $worktree " -n 1 -r
if [[ $REPLY =~ ^[Yy]$ ]]
then

    # Commit changes
    git commit

    # Push
    git push -u origin gh-pages

    echo "done"
    chmod -R +w "$worktree"
    git worktree remove "$worktree"
else
    chmod -R +w "$worktree"
    git worktree remove --force "$worktree"
fi
