 for d in tests/data/*; do
   pushd "$d"
   cat "options.txt" | xargs cargo run -- "input.sgf" -o "output.svg"
   popd
 done
