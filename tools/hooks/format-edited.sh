#!/bin/sh
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$PATH"
file=$(jq -r '.tool_input.file_path // empty')
case "$file" in
  *.rs) rustfmt --edition 2024 "$file" ;;
  *.typ) typstyle -i "$file" ;;
esac
exit 0
