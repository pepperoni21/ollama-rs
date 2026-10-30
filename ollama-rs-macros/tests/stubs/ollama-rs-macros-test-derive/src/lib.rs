//! No-op derive macros used by the macro expansion tests.
//!
//! The generated parameter structs derive `serde::Deserialize` and
//! `schemars::JsonSchema`. Expanding the real derives would add thousands of
//! lines to the snapshots, so the test stub re-exports these instead.

use proc_macro::TokenStream;

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn deserialize(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(JsonSchema, attributes(schemars))]
pub fn json_schema(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
