#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
//! The code in this file was shamelessly stolen from
//! https://stackoverflow.com/questions/71541765/rust-async-drop

#[async_trait::async_trait]
pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

#[cfg(feature = "no-default-bound")]
mod no_default_bound;
#[cfg(feature = "no-default-bound")]
pub use no_default_bound::AsyncDropper;

#[cfg(not(feature = "no-default-bound"))]
mod default;
#[cfg(not(feature = "no-default-bound"))]
pub use default::AsyncDropper;
