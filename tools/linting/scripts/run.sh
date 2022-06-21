#!/bin/bash
set -euo pipefail

THIS_DIR=$(realpath "$(dirname "${BASH_SOURCE[0]}")")
PROJECT_DIR=$(dirname "$THIS_DIR")

# shellcheck source=/dev/null
[[ "${HERMIT_ENV:-}" == "$PROJECT_DIR" ]] || source "$PROJECT_DIR/bin/activate-hermit"

(
  # Run setup first
  "$THIS_DIR/setup.sh"
)

(
  # Then run specific linters
  "$THIS_DIR/_bash.sh"
  "$THIS_DIR/_cspell.sh"
  "$THIS_DIR/_markdown.sh"
  "$THIS_DIR/_prettier.sh"
)

printf "\n\n✅ Lint Success ✅\n\n\n"