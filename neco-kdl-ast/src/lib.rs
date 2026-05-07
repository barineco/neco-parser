#![doc = include_str!("../README.md")]

mod convention;
mod cross_ref;
mod document;
mod nsid;
mod structured;
mod transform;

pub use convention::{Convention, Marker};
pub use cross_ref::{CrossRef, CrossRefParseError};
pub use document::{Document, LayoutMode, LayoutViolation, LayoutViolationKind};
pub use nsid::NsidPath;
pub use structured::{StructuredFacade, StructuredName, StructuredNode};
pub use transform::TransformOutcome;

pub mod kdl {
    pub use neco_kdl::{
        parse, serialize, KdlDocument, KdlEntry, KdlError, KdlErrorKind, KdlNode, KdlNumber,
        KdlValue,
    };
}
