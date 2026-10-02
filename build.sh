#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")" && pwd)"
cd "$root"

usage() {
  cat <<'EOF'
Build the complete project for production.

Usage:
  ./build.sh [--single | --separate]

Options:
  --single    embed the frontend in the server binary
  --separate  keep the server binary and frontend files separate
  -h, --help  show this help

With no option, an interactive wizard asks which output to build.
EOF
}

mode=""
case "${1:-}" in
  --single) mode="single" ;;
  --separate) mode="separate" ;;
  -h|--help) usage; exit 0 ;;
  "") ;;
  *) echo "error: unknown option '$1'" >&2; usage >&2; exit 2 ;;
esac

if [[ $# -gt 1 ]]; then
  echo "error: expected at most one option" >&2
  usage >&2
  exit 2
fi

has_frontend=false
if [[ -d apps/web ]]; then
  has_frontend=true
fi

if [[ "$has_frontend" == true && -z "$mode" ]]; then
  echo "Production build:"
  echo "  1) single    embed the frontend in the server binary"
  echo "  2) separate  keep the server binary and frontend files separate"
  printf "Choose [1]: "
  read -r answer
  case "${answer:-1}" in
    1|single) mode="single" ;;
    2|separate) mode="separate" ;;
    *) echo "error: enter 1, 2, single or separate" >&2; exit 2 ;;
  esac
fi

if [[ "$has_frontend" == false ]]; then
  mode="api-only"
fi

if [[ -f apps/web/package-lock.json ]]; then
  echo "==> Building frontend"
  (cd apps/web && npm ci && npm run build)
fi

echo "==> Building server"
case "$mode" in
  single)
    cargo build --release --locked --features embed-web
    echo "Built a single executable in target/release/."
    ;;
  separate)
    cargo build --release --locked --no-default-features
    if [[ -d apps/web/dist ]]; then
      web_dir="apps/web/dist/"
    else
      web_dir="apps/web/"
    fi
    echo "Built the server in target/release/. Deploy it with $web_dir"
    ;;
  api-only)
    cargo build --release --locked
    echo "Built the API server executable in target/release/."
    ;;
esac
