```rust
use neco_kdl::parse;
let doc = parse("name \"alice\"\n").unwrap();
assert_eq!(doc.nodes()[0].first_string_arg(), Some("alice"));
```
