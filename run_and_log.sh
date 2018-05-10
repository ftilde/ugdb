#! /bin/sh
output="/tmp/output.log"
RUST_BACKTRACE=1 cargo run -- "$@" 2> $output
cat $output
rm $output
