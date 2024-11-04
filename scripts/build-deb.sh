#!/usr/bin/env sh

set -e

scripts/generate-man.sh 1>&2
cd "$(git rev-parse --show-toplevel)"
cargo deb --no-build
