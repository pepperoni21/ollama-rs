//! Minimal stand-in for `ollama-rs` used by the macro expansion tests.
//!
//! The `#[function]` macro emits paths into `ollama_rs`, so macrotest's
//! generated crate needs a crate with that name. Using the real crate would
//! pull serde/schemars derive expansions into the snapshots; this stub only
//! needs to make those paths resolvable.

pub mod re_exports {
    pub mod serde {
        pub use ollama_rs_macros_test_derive::Deserialize;
    }

    pub mod schemars {
        pub use ollama_rs_macros_test_derive::JsonSchema;
    }
}

pub mod generation {
    pub mod tools {
        use std::future::Future;

        pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

        pub trait Tool: Send + Sync {
            type Params;

            fn name() -> &'static str;
            fn description() -> &'static str;
            fn call(
                &mut self,
                parameters: Self::Params,
            ) -> impl Future<Output = Result<String>> + Send;
        }
    }
}
