#!/bin/sh
set -xe

rm -rf build
yarn run build
git add -f build
git stash
git co precompiled
git rm -r build
git stash pop
git commit -am "Precompiled JS."
git push -u origin precompiled

