## What is it?
An (INCOMPLETE) awk compiler backed by LLVM.

## How to use

```shell
## Build the runtime.cpp library which will be linked with your awk program
./run.sh 
## Run your awk program from a file
cargo run test.awk data.txt
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