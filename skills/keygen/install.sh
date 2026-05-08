#!/usr/bin/env bash
# Install the keygen-cli Skill into ~/.claude/skills (or $CLAUDE_SKILLS_DIR).
#
# Author: okooo5km

set -euo pipefail

SRC="$(cd "$(dirname "$0")" && pwd)"
DEST="${CLAUDE_SKILLS_DIR:-$HOME/.claude/skills}"
TARGET="$DEST/keygen"

mkdir -p "$DEST"

if [[ -L "$TARGET" || -e "$TARGET" ]]; then
  echo "Skill already present at $TARGET — removing old link/dir."
  rm -rf "$TARGET"
fi

ln -s "$SRC" "$TARGET"
echo "✓ keygen-cli skill linked: $TARGET → $SRC"
echo "  Restart Claude Code (or run /reload) to pick up the new skill."

if ! command -v keygen >/dev/null 2>&1; then
  echo
  echo "⚠  'keygen' binary not found in PATH."
  echo "   See $SRC/references/installation.md for install options."
fi
