```rust
use neco_kdl::KdlNode;
let n = KdlNode::builder("greet").string_arg("hello").build();
assert_eq!(n.name(), "greet");
assert_eq!(n.first_string_arg(), Some("hello"));
```
