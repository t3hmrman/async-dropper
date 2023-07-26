// re-exports for the macro crate to use
pub use anyhow;
pub use thiserror;
#[cfg(feature = "tokio")]
pub use tokio;
pub use async_dropper_derive as derive;
pub use async_dropper_simple as simple;
