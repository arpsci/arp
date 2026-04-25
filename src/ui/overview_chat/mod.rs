// NOTE: This module is in transition from prototype to crate-internal API.
// Keep exports explicit here while downstream UI modules are migrated.
#![allow(dead_code, unused_imports)]

pub use incoming::{MessageSource, should_dispatch_to_model};
pub mod chat;
pub use chat::*;
pub mod incoming;
pub mod store;
pub use store::*;
pub mod audit;
pub mod ollama;
