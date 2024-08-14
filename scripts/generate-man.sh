#!/usr/bin/env sh
#
# Generate the man page using help2man

repo_root="$(git rev-parse --show-toplevel)"
man_dir="${repo_root}/man"
cargo build --release
binary="${repo_root}/target/release/sgf-render"

help2man --locale=en --no-info --include=${man_dir}/sgf-render.1.in "$binary" | preconv | gzip -9 > "${man_dir}/sgf-render.1.gz"
