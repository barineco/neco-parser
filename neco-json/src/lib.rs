//! Zero-dependency minimal JSON codec for no_std environments.
//!
//! See the owned and borrowed read APIs on this crate's public types.

#![no_std]
extern crate alloc;

mod borrowed;
mod borrowed_string;
mod decode_char;
mod encode;
mod error;
mod json_string;
mod number;
mod parse;
mod scan;
mod traits;
mod value;

pub use borrowed::{parse_borrowed, JsonArrayItems, JsonKind, JsonObjectEntries, JsonRef};
pub use borrowed_string::StringDecodeError;
pub use encode::encode;
pub use error::{AccessError, EncodeError, ParseError, ParseErrorKind};
pub use json_string::JsonString;
pub use number::JsonNumber;
pub use parse::parse;
pub use traits::{FromJson, ToJson};
pub use value::JsonValue;
