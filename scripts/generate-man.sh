#!/usr/bin/env sh
#
# Generate the man page using help2man

set -e

repo_root="$(git rev-parse --show-toplevel)"
infile="${repo_root}/man/sgf-render.1.in"
outfile="${repo_root}/man/sgf-render.1.gz"
cargo_output=$(cargo build --message-format=json-render-diagnostics --release)
binary=$(echo "$cargo_output" | jq -js '[.[] | select(.reason == "compiler-artifact") | select(.target.name = "sgf-render")] | last | .executable')

help2man --locale=en --no-info --include=${infile} "$binary" | preconv | gzip -9 > "${outfile}"
