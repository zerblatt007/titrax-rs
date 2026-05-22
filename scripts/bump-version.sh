#!/usr/bin/env bash
# bump-version.sh — Bump the version in Cargo.toml
# Usage: ./scripts/bump-version.sh [patch|minor|major]
#
# Arguments:
#   patch  — increment the patch version (0.0.x)
#   minor  — increment the minor version (0.x.0), reset patch to 0
#   major  — increment the major version (x.0.0), reset minor and patch to 0
set -euo pipefail

LEVEL="${1:-patch}"
CARGO_TOML="$(dirname "$0")/../Cargo.toml"

current=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r major minor patch <<<"$current"

case "$LEVEL" in
patch) patch=$((patch + 1)) ;;
minor)
	minor=$((minor + 1))
	patch=0
	;;
major)
	major=$((major + 1))
	minor=0
	patch=0
	;;
*)
	echo "Usage: $0 [patch|minor|major]"
	exit 1
	;;
esac

new_version="${major}.${minor}.${patch}"
sed -i "s/^version = \"${current}\"/version = \"${new_version}\"/" "$CARGO_TOML"
echo "Bumped version: ${current} → ${new_version}"
