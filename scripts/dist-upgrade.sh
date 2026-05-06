#!/usr/bin/env bash
# Upgrade cargo-dist and update CI workflows.
# Patches only the dist installer URL in release.yml instead of full regeneration,
# preserving custom jobs (publish-crates, update-homebrew, announce).
set -euo pipefail

CONFIG="dist-workspace.toml"
WORKFLOW=".github/workflows/release.yml"

if [ $# -eq 0 ]; then
  echo "Usage: $0 <version>  (e.g. $0 0.32.0)"
  exit 1
fi

VERSION="$1"

if [ ! -f "$CONFIG" ]; then
  echo "Error: $CONFIG not found"
  exit 1
fi

echo "==> Upgrading cargo-dist to v${VERSION}..."

# Step 1: Update the version in config
sed -i.bak "s/^cargo-dist-version = \".*\"/cargo-dist-version = \"${VERSION}\"/" "$CONFIG"
rm -f "${CONFIG}.bak"

# Step 2: Install new dist version
cargo install cargo-dist --version "^${VERSION}" --locked --force

# Step 3: Regenerate MSI template (safe, no custom content)
dist generate --mode msi

# Step 4: Patch the dist installer URL in release.yml
# The installer URL contains the version, e.g.:
#   .../cargo-dist/releases/download/v0.31.0/cargo-dist-installer.sh
# We update it to the new version while keeping everything else intact.
if [ -f "$WORKFLOW" ]; then
  sed -i.bak "s|cargo-dist/releases/download/v[^/]*/cargo-dist-installer.sh|cargo-dist/releases/download/v${VERSION}/cargo-dist-installer.sh|" "$WORKFLOW"
  rm -f "${WORKFLOW}.bak"
  echo "    Patched installer URL in release.yml"
else
  echo "    Warning: $WORKFLOW not found, skipping patch"
fi

echo "==> Done. Review changes with: git diff"
echo "    Config version: v${VERSION}"
echo "    release.yml: installer URL patched, custom jobs preserved"
