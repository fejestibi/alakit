pub mod core;
pub mod dom;
pub mod engine;
pub mod store;

// Re-exportáljuk a leggyakoribb API-kat
pub use crate::core::{AlakitController, AppContext, ControllerRegistration};
pub use crate::dom::{Element, RustDOM};
pub use crate::engine::AlakitEngine;
pub use crate::store::Store;
