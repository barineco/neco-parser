```rust
use neco_kdl::{KdlDocument, KdlNode};
let doc = KdlDocument::builder()
    .node(KdlNode::builder("alpha").build())
    .node(KdlNode::builder("beta").build())
    .build();
assert_eq!(doc.nodes().len(), 2);
```
