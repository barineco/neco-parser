```rust
use neco_kdl::parse;
let doc = parse("parent {\n    item \"a\"\n    item \"b\"\n}\n").unwrap();
let parent = &doc.nodes()[0];
assert_eq!(parent.find_child("item").and_then(|c| c.first_string_arg()), Some("a"));
```
