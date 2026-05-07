//! Zero dependency KDL v2 parser, serializer, and document builder.

pub mod builder;
mod comment;
pub mod convert;
pub mod normalize;
mod number;
mod parse;
mod scan;
mod serialize;
mod string;
mod value;

pub use builder::{KdlDocumentBuilder, KdlNodeBuilder};
pub use convert::{kdl_document_to_value, value_to_kdl_document, Value};
pub use normalize::normalize;
pub use parse::parse;
pub use serialize::serialize;
pub use value::{KdlDocument, KdlEntry, KdlError, KdlErrorKind, KdlNode, KdlNumber, KdlValue};
