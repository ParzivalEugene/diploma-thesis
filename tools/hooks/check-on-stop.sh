#!/bin/sh
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$PATH"
cd "$CLAUDE_PROJECT_DIR" || exit 0
git status --porcelain | grep -Eq '\.(rs|typ)$|Cargo\.toml$' || exit 0
if ! output=$(just check-fast 2>&1); then
  printf '%s\n' "$output" | tail -40 >&2
  exit 2
fi
exit 0
