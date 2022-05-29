
```shell
# produces /tmp/crawk.bc llvm bitcode from awk input
cargo run test.awk 

# bitcode -> native assembly
/Users/n8ta/llvm-13/bin/llc /tmp/crawk.bc -o crawk.s

# link rust based dynamic lib with my print function with assembly
clang crawk.s runtime/target/release/libruntime.dylib
```

## Todo
- Range patterns `pattern1, pattern2 { print $0 }` Matches from the first line matching pattern1 to the next line matching pattern 2
- Regex patterns `/abc/ { print $0 }`
- Fields `{ print $1 }`
- `awk '{ tmp = $1; $1 = $2; $2 = tmp; print $0 } ' data.txt`  
assigning to fields
- Assignment to the field separated `BEGIN { FS = "\t" }`
- Matched by and not matched by operators `~` and `!~`
 eg.  `expr ~ /regex/ { print $0 }` prints when the string value of expr matches regexp.  
- String `>=` comparisons `$0 >= "M"` matches all lines that begin with M N O ...
- If without else
- break, continue, do while, next, exit, exit expression, `for var in array`, `for (expr; expr; expr) stmts`
printf, print `expression-list`
  - 