#!/usr/bin/env bash
# validate-worklog.sh — Validates worklog YAML front matter format
# Usage: ./scripts/validate-worklog.sh [worklog-file]
#   If no file is given, validates the most recently modified worklog.
set -euo pipefail

FILE="${1:-}"
if [[ -z "$FILE" ]]; then
	FILE=$(ls -t /work/docs/worklogs/*.md 2>/dev/null | head -1)
fi

if [[ -z "$FILE" || ! -f "$FILE" ]]; then
	echo "ERROR: No worklog file found"
	exit 1
fi

echo "Validating: $FILE"

# Check front matter opening ---
if ! head -1 "$FILE" | grep -q '^---$'; then
	echo "ERROR: Missing YAML front matter opening ---"
	exit 1
fi

# Check all required keys are present
for key in when why what model tags; do
	if ! grep -q "^${key}:" "$FILE"; then
		echo "ERROR: Missing required key: ${key}"
		exit 1
	fi
done

# Extract front matter (lines between first and second ---)
front_matter=$(awk '/^---$/{if(++c==2)exit} c==1' "$FILE")

# Reject any keys not in the allowed set
forbidden=$(echo "$front_matter" | grep -E '^[a-z_]+:' | grep -vE '^(when|why|what|model|tags):' || true)
if [[ -n "$forbidden" ]]; then
	echo "ERROR: Forbidden keys in front matter:"
	echo "$forbidden"
	exit 1
fi

echo "✓ Worklog validation passed: $FILE"
exit 0
