# rot
Rust compiler for the the Brainfuck variant for the brain-rotted

## Usage
```
./rot hello.rot # run program in interpreter mode
./rot -n -o hello hello.rot # compile program into x86-64 executable
./rot -na -o hello.asm hello.rot # output assembly instead of linking
```

## Syntax 
| Command | Brainfuck Equivalent | Explanation
| --- | --- | --- |
| 676 | > | Move data pointer one cell right | 
| 767 | < | Move data pointer one cell left |
| 67 | + | Increment current cell under the data pointer |
| 76 | - | Decrement current cell under the data pointer |
| 667 | . | Output the ASCII value of the current cell |
| 776 | , | Set the current cell to a single user-provided character |
| 6677 | [ | Jump to the closest 7766 below self if the current cell is 0, continuing otherwise |
| 7766 | ] | Jump to the closet 6677 instruction above self |
