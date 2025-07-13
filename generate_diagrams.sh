#!/bin/bash
set -exuo pipefail
cd "$(dirname "$0")"

rm -rv "./docs/diagrams" || true
mkdir -p ./docs/diagrams

pushd "./angel-sm"
  for state in $(RUST_LOG=error cargo run --features visualize --bin visualize -- list); do
    cargo run --features visualize --bin visualize -- graph -f png -o ../docs/diagrams/$state.png $state
  done
popd
