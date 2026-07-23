#!/usr/bin/env bash
# Server stub (A02): print subject-matching usage until Rust CLI exists (S01).
# Usage strings aligned with docs/raw/audit.md (AQ01) and RQ17 flags.
set -euo pipefail

usage() {
  cat <<'EOF'
 Usage: ./server -p <port> -x <width> -y <height> -n <team> [<team>] [<team>] ... -c <nb> [-t <t>]
 -p port number
 -x world width
 -y world height
 -n team_name_1 team_name_2 ...
 -c number of clients authorized at the beginning of the game
 -t [100] time unit divider (the greater t is, the faster the game will go)
EOF
}

# Stub: always print usage. Full arg handling is S01.
usage
exit 1
