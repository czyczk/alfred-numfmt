#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release
cp target/release/numfmt workflow/
chmod +x workflow/numfmt
cd workflow
zip -qry ../numfmt.alfredworkflow . -x '.*'
echo "Packed: numfmt.alfredworkflow"
