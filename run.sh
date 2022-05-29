#!/bin/sh
set -e
export RUST_BACKTRACE=1
cargo run "$1"
/Users/n8ta/llvm-13/bin/llc /tmp/crawk.bc -o /tmp/crawk.s
clang /tmp/crawk.s /Users/n8ta/code/crawk/runtime/target/release/libruntime.dylib -o /tmp/a.out
chmod +x /tmp/a.out

echo "Running final binary"
/tmp/a.out
