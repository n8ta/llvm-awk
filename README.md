```shell
# produces /tmp/crawk.bc llvm bitcode from awk input
cargo run test.awk 

# bitcode -> native assembly
/Users/n8ta/llvm-13/bin/llc /tmp/crawk.bc -o crawk.s

# link rust based dynamic lib with my print function with assembly
clang crawk.s runtime/target/release/libruntime.dylib
```