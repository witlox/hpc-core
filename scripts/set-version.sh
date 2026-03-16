#!/usr/bin/env bash
# set-version.sh — Computes release versions for all workspace crates.
#
# Each crate's patch version = git commit count + per-crate offset
# (see version-offsets.conf for crates migrated from other repos).
#
# Usage:
#   ./scripts/set-version.sh          # patches files in-place
#   ./scripts/set-version.sh --dry-run # prints computed versions only
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OFFSETS_FILE="$REPO_ROOT/scripts/version-offsets.conf"

# 1. Read base version from the workspace source of truth
BASE_VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml" | head -1)
if [ -z "$BASE_VERSION" ]; then
    echo "error: could not read version from Cargo.toml" >&2
    exit 1
fi

# 2. Extract year.major prefix (everything before the last dot)
PREFIX="${BASE_VERSION%.*}"

# 3. Compute commit count
COMMIT_COUNT=$(git -C "$REPO_ROOT" rev-list --count HEAD)

# Helper: look up offset for a crate name from the offsets file
get_offset() {
    local name="$1"
    if [ -f "$OFFSETS_FILE" ]; then
        grep -E "^${name}=" "$OFFSETS_FILE" 2>/dev/null | head -1 | cut -d= -f2 || echo 0
    else
        echo 0
    fi
}

DRY_RUN=false
if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN=true
else
    echo "Setting versions (commit count=$COMMIT_COUNT):"
fi

# 4. Compute and apply per-crate versions
for manifest in "$REPO_ROOT"/crates/*/Cargo.toml; do
    CRATE_NAME=$(sed -n 's/^name = "\(.*\)"/\1/p' "$manifest" | head -1)
    OFFSET=$(get_offset "$CRATE_NAME")
    [ -z "$OFFSET" ] && OFFSET=0
    PATCH=$((COMMIT_COUNT + OFFSET))
    CRATE_VERSION="${PREFIX}.${PATCH}"

    if [ "$DRY_RUN" = true ]; then
        echo "$CRATE_NAME=$CRATE_VERSION"
    else
        echo "  $CRATE_NAME: $BASE_VERSION -> $CRATE_VERSION"
        sed -i.bak "s/^version\.workspace = true/version = \"${CRATE_VERSION}\"/" "$manifest"
    fi
done

if [ "$DRY_RUN" = true ]; then
    exit 0
fi

# 5. Update Cargo.lock
cargo generate-lockfile --manifest-path "$REPO_ROOT/Cargo.toml" 2>/dev/null || true

# Clean up sed backup files
find "$REPO_ROOT" -name '*.bak' -delete

echo "Done"
