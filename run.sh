#!/bin/sh
set -e
echo "Compiling runtime.c into runtime.bc which will be linked by main rust program"
echo "it is very hard to get rust to produce static binaries so I have been reduced to this"
#export RUST_BACKTRACE=1
#cargo run "$1"
#/Users/n8ta/llvm-13/bin/llc /tmp/crawk.bc -o /tmp/crawk.s
#clang /tmp/crawk.s /Users/n8ta/code/crawk/runtime/target/release/libruntime.dylib -o /tmp/a.out
#chmod +x /tmp/a.out
#
#echo "Running final binary"
#/tmp/a.out
#
## clang -S -emit-llvm runtime.c -static -o runtime.bc
## clang -emit-llvm -c runtime.c -static -o runtime.bc
clang -emit-llvm -c runtime.c -static -o runtime.bc
