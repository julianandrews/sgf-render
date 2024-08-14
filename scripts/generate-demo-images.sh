#!/usr/bin/env sh
#
# Generate the demo images.

repo_root="$(git rev-parse --show-toplevel)"
cargo build --release
binary="${repo_root}/target/release/sgf-render"
data_dir="${repo_root}/tests/data"
demo_dir="${repo_root}/demo"

$binary ${data_dir}/prob45/input.sgf --shrink-wrap --width 400 -o ${demo_dir}/prob45.svg
$binary ${data_dir}/prob45/input.sgf --shrink-wrap -o ${demo_dir}/prob45.png
$binary ${data_dir}/prob45/input.sgf --shrink-wrap --style fancy -o ${demo_dir}/prob45-fancy.svg
$binary ${data_dir}/markup/input.sgf -n 2 --range cc-qk -o ${demo_dir}/markup.svg
$binary ${data_dir}/numbered_moves/input.sgf -n 120 --move-numbers -o ${demo_dir}/simple-numbered.svg
$binary ${data_dir}/numbered_moves/input.sgf --style minimalist -n 120 --move-numbers -o ${demo_dir}/minimalist-numbered.svg
$binary ${data_dir}/custom_style/input.sgf -n 120 --move-numbers=80 --custom-style ${data_dir}/custom_style/wacky.toml -o ${demo_dir}/wacky.svg
$binary ${data_dir}/kifu/input.sgf --kifu --style minimalist -o ${demo_dir}/kifu.svg
