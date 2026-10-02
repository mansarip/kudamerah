#!/usr/bin/env bash
# Generates every frontend × example combination from the working tree. For each one, it
# checks for leftover template tokens and runs fmt, clippy and tests, plus the npm build
# for solid.
#
# Usage: starter/test.sh [out-dir]     SKIP_NPM=1 skips the SolidJS npm steps.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$(mktemp -d)}"
export CARGO_TARGET_DIR="$out/target" # shared, so dependencies compile once

cargo test --quiet --manifest-path "$root/Cargo.toml" -p starter
cargo build --quiet --manifest-path "$root/Cargo.toml" -p starter
wizard="$CARGO_TARGET_DIR/debug/starter"

for frontend in vanilla htmx solid none; do
  for example in example no-example; do
    name="$frontend-$example"
    dir="$out/$name"
    echo "==> $name"
    rm -rf "$dir"
    "$wizard" new "$dir" --frontend "$frontend" "--$example" --no-git --yes >/dev/null

    if grep -rIni --exclude-dir=node_modules -e kudamerah -e 'starter:' "$dir"; then
      echo "leftover template tokens in $name" >&2
      exit 1
    fi
    (cd "$dir" && cargo fmt --check && cargo clippy --quiet --all-targets -- -D warnings && cargo test --quiet)

    if [[ "$frontend" == solid && -z "${SKIP_NPM:-}" ]]; then
      (cd "$dir/apps/web" && npm ci --silent && npm run --silent check && npm run --silent build)
    fi
  done
done

echo "All variants OK ($out)"
