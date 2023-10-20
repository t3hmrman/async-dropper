//! `async_dropper` provides two ad-hoc implementations of asynchronous `drop` behavior (`AsyncDrop`).
//!
//! - `async_dropper::AsyncDropper` is a wrapper struct (suggested on [StackOverflow by paholg](https://stackoverflow.com/a/75584109))
//! - `async_dropper::AsyncDrop` is a `derive` macro which generates code that enables async drop functionality to run during a regular `drop`. That code requires `T: Default + PartialEq + Eq`.
//!
//! Here is a quick example of the shorter `async_dropper::AsyncDropper`:
//!
//! ```ignore
//! // NOTE: you must have the 'simple' feature enabled to use this code!
//!
//! /// This object will be async-dropped (which must be wrapped in AsyncDropper)
//! #[derive(Default)]
//! struct AsyncThing(String);
//!
//! #[async_trait]
//! impl AsyncDrop for AsyncThing {
//!     async fn async_drop(&mut self) {
//!         tokio::time::sleep(Duration::from_secs(2)).await; // fake work
//!         Ok(())
//!     }
//! }
//!
//! #[your_async_runtime_of_choice::main] // i.e. tokio::main or async_std::main
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     drop(AsyncDropper::new(AsyncThing(String::from("test"))));
//!     Ok(())
//! }
//! ```
//!
//! Note that `AsyncThing` must be wrapped in `async_dropper::AsyncDropper`.
//!
//! For `async_dropper::AsyncDrop`, the simplest example looks like this:
//!
//! ```ignore
//! // NOTE: you must have the 'derive' feature enabled to use this code!
//!
//! use async_dropper::AsyncDrop;
//!
//! // Your struct (named field structs and tuple structs both work)
//! #[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
//! struct AsyncThing(String);
//!
//! /// How it drops, asynchrounously
//! #[async_trait]
//! impl AsyncDrop for AsyncThing {
//!     async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
//!         tokio::time::sleep(Duration::from_secs(2)).await; // fake work
//!         Ok(())
//!     }
//! }
//!
//! #[your_async_runtime_of_choice::main] // i.e. tokio::main or async_std::main
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     drop(AsyncThing(String::from("test")));
//!     Ok(())
//! }
//! ```
//!
//! The `async_dropper::AsyncDrop` is interesting because it attempts to automatically (if somewhat painfully) determine
//! whether an object should have the defined asynchronous drop behavior performed by checking whether it is equal
//! to `Self::default()`.
//!
//! **Said differently, async drop behavior is skipped if an instance of `T` is exactly equal to a `T::default()`**.
//!
//! For convenience, a `reset(&mut self)` function that sets any T to T::default() is automatically derived, but it can be overriden via `AsyncDrop#reset()`.
//!
//! If `T` is *not* exactly equal to `T::default()`, the assumption is that `T` must still be holding things that require asynchronous dropping behavior, and as such that behavior will be performed.
//!
//! **If `reset(&mut self)` does not return `T` to a state where it is equal to `T::default()`, `drop` will panic**

/// Re-export #[derive(AsyncDrop)]
#[cfg(feature = "derive")]
extern crate async_dropper_derive;
#[cfg(feature = "derive")]
pub use async_dropper_derive::AsyncDrop;

#[cfg(feature = "simple")]
pub use async_dropper_simple::{AsyncDropper, AsyncDrop};

#[cfg(all(feature = "simple", feature = "derive"))]
compile_error!("both 'derive' and 'simple' features cannot be enabled at the same time");

#[cfg(all(not(feature = "simple"), not(feature = "derive")))]
compile_error!("either the 'derive' feature or the 'simple' feature must be enabled");

#[derive(Debug)]
pub enum AsyncDropError {
    UnexpectedError(Box<dyn std::error::Error>),
    Timeout,
}

/// What to do when a drop fails
#[derive(Debug, PartialEq, Eq)]
pub enum DropFailAction {
    // Ignore the failed drop
    Continue,
    // Elevate the drop failure to a full on panic
    Panic,
}

/// Types that can reset themselves to T::default()
pub trait ResetDefault {
    fn reset_to_default(&mut self);
}

/// The operative trait that enables AsyncDrop functionality.
/// Normally, implementing only async_drop(&mut self) and reset(&mut self) is necessary.
#[async_trait::async_trait]
#[cfg(feature = "derive")]
pub trait AsyncDrop: Default + PartialEq + Eq + ResetDefault {
    /// Operative drop that does async operations, returning
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        Ok(())
    }

    /// A way to reset the object (set all it's internal members to their default).
    /// This method is used after a successful AsyncDrop, to ensure that future drops do not
    /// perform async_drop behavior twice.
    fn reset(&mut self) {
        self.reset_to_default();
    }

    /// Timeout for drop operation, meant to be overriden if needed
    fn drop_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(3)
    }

    /// What to do what a drop fails
    fn drop_fail_action(&self) -> DropFailAction {
        DropFailAction::Continue
    }
}
