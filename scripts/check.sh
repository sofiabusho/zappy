#!/usr/bin/env bash
# Run lint/format/test gates for server, client, gui, and scripts (A03 / G0).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

section() {
  printf '\n==> %s\n' "$1"
}

# Local fallbacks for ruff/pytest when not installed system-wide.
"${ROOT}/scripts/ensure-dev-tools.sh"
export PATH="${ROOT}/.tools/bin:${PATH:-}"
if [[ -d "${ROOT}/.tools/py" ]]; then
  export PYTHONPATH="${ROOT}/.tools/py${PYTHONPATH:+:${PYTHONPATH}}"
fi

section "server: cargo fmt --check / clippy / test"
(
  cd server
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
)

section "client: ruff / pytest"
(
  cd client
  if command -v ruff >/dev/null 2>&1; then
    ruff check .
  elif python3 -m ruff --version >/dev/null 2>&1; then
    python3 -m ruff check .
  else
    echo "error: ruff not found after ensure-dev-tools" >&2
    exit 1
  fi
  if command -v pytest >/dev/null 2>&1; then
    pytest
  else
    python3 -m pytest
  fi
)

section "gui: npm run lint / npm test"
(
  cd gui
  if [[ ! -d node_modules ]]; then
    echo "gui: node_modules missing; running npm install"
    npm install
  fi
  npm run lint
  npm test
)

section "scripts: scaffold + wrapper + harness + readme tests"
python3 scripts/test_scaffold.py
python3 scripts/test_wrappers.py
python3 scripts/test_check_harness.py
python3 scripts/test_readme_quickstart.py

printf '\nOK: all checks passed\n'
