# mustdown

`mustdown` is an experimental implementation of [CommonMark](https://commonmark.org/).

# Status
- Currently only support `ASCII` code.
- Table not supported.

# Usage
```rust
extern crate mustdown;
use mustdown::Parser;
let mut parser = Parser::new();
let result = parser.parse("**hello world**");
assert_eq!(result, "<p>\n<strong>hello world</strong>\n</p>");
```
