#!/usr/bin/env sh
cargo run -- tests/data/prob45/input.sgf --shrink-wrap --width 400 -o demo/prob45.svg
cargo run -- tests/data/prob45/input.sgf --shrink-wrap -o demo/prob45.png
cargo run -- tests/data/prob45/input.sgf --shrink-wrap --style fancy -o demo/prob45-fancy.svg
cargo run -- tests/data/markup/input.sgf -n 2 --range cc-qk -o demo/markup.svg
cargo run -- tests/data/numbered_moves/input.sgf -n 120 --move-numbers -o demo/simple-numbered.svg
cargo run -- tests/data/numbered_moves/input.sgf --style minimalist -n 120 --move-numbers -o demo/minimalist-numbered.svg
cargo run -- tests/data/custom_style/input.sgf -n 120 --move-numbers --first-move-number 80 --custom-style tests/data/custom_style/wacky.toml -o demo/wacky.svg
