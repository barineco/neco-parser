# neco-kdl-ast

[日本語](README-ja.md)

necosystems series structured AST layer for KDL v2 documents.

It builds namespace path, cross-reference, structured naming, and procedural nesting abstractions on top of the `neco-kdl` parser, and implements the shared `neco-ast` structured access traits.

The crate has two layers of behavior. The reading side exposes accessors over `KdlDocument` that interpret namespace paths, type annotations, property-vs-child duality, and form X / form Y kind keywords without modifying the document. The transformation side accepts a consumer-supplied `Convention` of marker reserved words or prefix characters and rewrites the document along five orthogonal containment axes, each of which is round-trippable.

## Usage

### Parse and read

```rust
use neco_kdl_ast::Document;

let input = r#"
    cratis "encoding" version=1 {
        provides {
            axiom "encoding.base64.encode"
            axiom "encoding.base64.decode"
        }
    }
"#;
let doc = Document::parse(input).unwrap();

for node in doc.structured_nodes() {
    println!("{}", node.kind());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Cross-reference parsing

```rust
use neco_kdl_ast::CrossRef;

let cr = CrossRef::parse("app.bsky.actor.defs#profileViewDetailed").unwrap();
assert_eq!(cr.nsid().display(), "app.bsky.actor.defs");
assert_eq!(cr.fragment(), Some("profileViewDetailed"));
```

### Transform with a consumer convention

```rust
use neco_kdl_ast::{Convention, Document, Marker};

let app_marker = Marker::Prefix(':');
let conv = Convention::new().with_marker(app_marker.clone());

let input = r#"(app)com.vscodium.codium-insiders { bindings {} }"#;
let doc = Document::parse(input).unwrap();
let (doc, _) = doc.expand_type_annotations(&app_marker, &conv);
let (doc, _) = doc.nest(&conv);
let (doc, _) = doc.collapse_type_annotations(&app_marker, &conv);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The marker boundary protects the immediate child of `:app` from namespace expansion, so the reverse-domain identifier `com.vscodium.codium-insiders` round-trips intact.

## API

### Core types

| Type | Description |
|------|-------------|
| `NsidPath` | Dot-separated namespace path with bidirectional FS path conversion |
| `CrossRef` | Reference of the form `<NsidPath>#<fragment>` covering three syntactic shapes |
| `CrossRefParseError` | Parse error variants for cross-reference strings |
| `StructuredName` | `{ kind: Option<NsidPath>, identifier: NsidPath }` over form X and form Y |
| `StructuredNode<'a>` | Borrowed view over `KdlNode` exposing kind, identifier, and procedural depth |
| `Document` | Owned wrapper over `KdlDocument` for indexed lookup, resolve, layout verification, and transforms |
| `Convention` | Consumer-supplied per-axis normal-form declaration plus reserved marker list |
| `Marker` | `Kind(String)` or `Prefix(char)` reserved name acting as a containment modifier |
| `AxisForm` | `Off` / `Expand` / `Collapse` / `ExpandWithMarker(Marker)` / `CollapseWithMarker(Marker)` for axes 1 / 2 / 4 / 5 |
| `PropertyChildForm` | `Off` / `Expand` / `Collapse` for axis 3 (fused-form-only by enum axiom) |

### Read accessors

| Method | Axis |
|--------|------|
| `node_name_as_nsid` | namespace |
| `dot_chain_depth` / `dot_chain_kind` | procedure |
| `attribute_str` / `attribute_bool` / `attribute_int` | property-child equivalence |
| `type_annotation` | type annotation |
| `structured_name` / `structured_name_form_x` / `structured_name_form_y` | kind keyword |

### Transforms (10 methods + render_as orchestration)

Each axis pairs one expand and one collapse. All transforms take `&Convention` and respect marker boundaries: the immediate child of any registered marker is preserved while deeper levels are processed normally.

| Axis | Expand | Collapse |
|------|--------|----------|
| namespace | `Document::nest` | `Document::flatten` |
| procedure | `Document::expand_dot_chain` | `Document::collapse_dot_chain` |
| property-child | `Document::expand_properties` | `Document::collapse_properties` |
| type annotation | `Document::expand_type_annotations` | `Document::collapse_type_annotations` |
| kind keyword | `Document::expand_kind_keyword` | `Document::collapse_kind_keyword` |

`Document::render_as(&Convention) -> Document` reads the per-axis normal-form declarations on the supplied convention and orchestrates the per-axis transforms in the fixed order axis 5 → 4 → 1 → 2 → 3 (marker-boundary-preserving). The default convention has every axis set to `Off`, so `render_as(&Convention::default())` is identity and existing read accessors retain backward-compatible behavior.

### Layout verification

| Type | Description |
|------|-------------|
| `LayoutMode` | `Strict1To1`, `Bundle`, or `CratisDir` |
| `LayoutViolation` and `LayoutViolationKind` | FS path / NSID consistency report |

### Shared structured access

| Item | Description |
|------|-------------|
| `neco_ast::StructuredNode<'a>` | Shared trait implemented by `StructuredNode<'a>` |
| `neco_ast::StructuredDocument<'a>` | Shared trait implemented by `Document` |
| `StructuredFacade<'a>` | Local five-method trait retained for existing KDL callers |

See [docs/INDEX.md](docs/INDEX.md) for the full reference and worked examples.

## License

MIT
