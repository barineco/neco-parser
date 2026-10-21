# neco-kdl-ast documentation index

[日本語](INDEX.md)

A structured AST layer built on top of the `neco-kdl` parser.

It provides the following:

- namespace paths
- cross-references
- structured naming
- procedural nesting
- an isomorphic transform across five axes covering property, type annotation, and kind keyword forms

It also provides KDL-specific accessors and a shared `neco-ast` trait implementation.

## Documents

| Document | Summary |
|---|---|
| [architecture/overview](architecture/overview-en.md) | Overall structure, the responsibilities of the parser and AST layers, main type list |
| [reference/syntax](reference/syntax-en.md) | Syntax and worked examples for structured naming, cross-reference, namespace nesting, and procedural nesting |

## By goal

| Goal | See |
|---|---|
| Understand the overall structure | [architecture/overview](architecture/overview-en.md) |
| Check the syntax and examples | [reference/syntax](reference/syntax-en.md) |
| Learn the difference between `.` ( namespace separator ) and leading `.` ( procedure depth ) | [reference/syntax § the two dot-based axes](reference/syntax-en.md) |
| Learn the isomorphic notations for property, type annotation, and kind keyword | [reference/syntax § five axes of isomorphic notation by containment](reference/syntax-en.md) |
| Learn the relation to the shared structured-access trait in `neco-ast` | [architecture/overview § shared structured access](architecture/overview-en.md) |
| Learn the mapping between FS path and namespace path | [reference/syntax § mapping to FS path](reference/syntax-en.md) |
| See an example of expressing an AT Protocol Lexicon in KDL | [reference/syntax § form Y and type annotation](reference/syntax-en.md) |
| Normalize all axes at once with `render_as` | [reference/syntax § Convention normal-form declaration](reference/syntax-en.md) |
| Learn the axis-OFF state and the fused-form axiom of `Convention` | [reference/syntax § Convention normal-form declaration](reference/syntax-en.md) |
