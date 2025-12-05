#!/usr/bin/env bash
set -euo pipefail

# usage: ./cut-new-release.sh <version>
# example: ./cut-new-release.sh 1.2.3

VERSION="$1"

# ensure working tree is clean
if [ -n "$(git status --porcelain)" ]; then
  echo "❌ Working directory not clean. Commit or stash changes first."
  exit 1
fi

# update Cargo.toml
echo "🔧 Updating Cargo.toml to version $VERSION..."
sed -i.bak "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

cargo test --all
cargo fmt --check
cargo clippy -- -D warnings

git add Cargo.toml Cargo.lock
git commit -m "chore(release): v$VERSION"
git tag "v$VERSION"

git push && git push --tags

echo "✅ Release v$VERSION cut successfully."
