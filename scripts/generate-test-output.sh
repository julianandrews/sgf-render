#!/usr/bin/env sh
#
# Update the SVG output for all integration tests to match current output.
#
# Run this when a change in rendering has modified the SVG output in some
# non-harmful way.

repo_root="$(git rev-parse --show-toplevel)"
cargo build --release
binary="${repo_root}/target/release/sgf-render"

for d in ${repo_root}/tests/data/*; do
    pushd "$d"
    cat "options.txt" | xargs "$binary" "input.sgf" -o "output.svg"
    popd
done
