# DN Kernel C Worker

A lightweight, offline C code analyzer using tree-sitter for parsing and implementing kernel coding style checks based on checkpatch.pl.

## Features

- **Offline**: No network dependencies, fully local analysis
- **Tree-sitter based**: Fast, accurate parsing using tree-sitter-c
- **Kernel style checks**: Implements 5 key rules from Linux kernel's checkpatch.pl

## Rules Implemented

1. **line-length**: Lines should not exceed 80 characters
2. **space-before-tab**: No spaces before tabs in indentation
3. **trailing-whitespace**: No trailing whitespace at end of lines
4. **keyword-spacing**: Space required after keywords (if, for, while, switch)
5. **brace-style**: Opening braces on same line (K&R style)

## Building

### Prerequisites

- GCC or Clang
- tree-sitter library
- tree-sitter-c grammar

### Install Dependencies (Ubuntu/Debian)

```bash
# Install tree-sitter
sudo apt-get install libtree-sitter-dev

# Clone and build tree-sitter-c
git clone https://github.com/tree-sitter/tree-sitter-c
cd tree-sitter-c
gcc -shared -o libtree-sitter-c.so -fPIC src/parser.c -I.
sudo cp libtree-sitter-c.so /usr/local/lib/
sudo ldconfig
```

### Build Worker

```bash
cd workers/c
make
```

## Usage

```bash
./dn-worker-c <file.c>
```

Output is JSON format:

```json
{
  "file": "example.c",
  "issues": [
    {
      "rule": "line-length",
      "severity": "warning",
      "message": "Line exceeds 80 characters (95 characters)",
      "line": 10,
      "column": 1
    }
  ]
}
```

## Integration with DN Kernel

Add to your profile configuration:

```toml
[profiles.kernel-c]
workers = ["c"]
rules = [
    "line-length",
    "space-before-tab",
    "trailing-whitespace",
    "keyword-spacing",
    "brace-style"
]
```

## License

Same as DN Kernel project.

## Example Output

Running on `test_sample.c`:

```bash
./dn-worker-c test_sample.c
```

Output:
```json
{
  "file": "test_sample.c",
  "issues": [
    {
      "rule": "line-length",
      "severity": "warning",
      "message": "Line exceeds 80 characters (104 characters)",
      "line": 3,
      "column": 1
    },
    {
      "rule": "space-before-tab",
      "severity": "error",
      "message": "Space before tab in indentation",
      "line": 7,
      "column": 2
    },
    {
      "rule": "keyword-spacing",
      "severity": "warning",
      "message": "Missing space after 'if' keyword",
      "line": 9,
      "column": 2
    }
  ]
}
```

## Notes

- This is a lightweight implementation using text-based parsing
- For more advanced AST-based analysis, tree-sitter integration can be added
- All checks are performed offline without any network dependencies
- Fast and efficient for large codebases
