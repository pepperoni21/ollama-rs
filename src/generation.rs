pub mod chat;
pub mod completion;
pub mod embeddings;
#[cfg_attr(docsrs, doc(cfg(feature = "function-calling")))]
#[cfg(feature = "function-calling")]
pub mod functions;
pub mod images;
pub mod options;
pub mod parameters;
