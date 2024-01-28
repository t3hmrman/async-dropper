#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
//! The code in this file was shamelessly stolen from
//! https://stackoverflow.com/questions/71541765/rust-async-drop

/// Represents an infallible case
pub static INFALLIBLE: &str = "INFALLIBLE";

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

#[async_trait::async_trait]
pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AsyncDropper<T: AsyncDrop + Send> {
    dropped: bool,
    timeout: Option<Duration>,
    inner: Option<T>,
}

impl<T: AsyncDrop + Send> AsyncDropper<T> {
    pub fn new(inner: T) -> Self {
        Self {
            dropped: false,
            timeout: None,
            inner: Some(inner),
        }
    }

    pub fn with_timeout(timeout: Duration, inner: T) -> Self {
        Self {
            dropped: false,
            timeout: Some(timeout),
            inner: Some(inner),
        }
    }

    /// Get a reference to the inner data.
    ///
    /// # Panics
    ///
    /// Can never occur since the Drop implementation only `take` the inner once, and only then.
    pub fn inner(&self) -> &T {
        self.inner.as_ref().expect(INFALLIBLE)
    }

    /// Get a mutable reference to the inner data.
    /// # Panics
    ///
    /// Can never occur since the Drop implementation only `take` the inner once, and only then.
    pub fn inner_mut(&mut self) -> &mut T {
        self.inner.as_mut().expect(INFALLIBLE)
    }
}

/// It doesn't require default, this tradeoff means that you need to use `AsyncDropper::new` or `AsyncDropper::with_timeout`.
/// Since `AsyncDropper::default()` doesn't create `T::default()` inner, we create a dummy Default instance om `std::mem::take` to be thrown away.
impl<T> Default for AsyncDropper<T>
where
    T: AsyncDrop + Send,
{
    fn default() -> Self {
        Self {
            dropped: true,
            timeout: None,
            inner: None,
        }
    }
}

impl<T> Deref for AsyncDropper<T>
where
    T: AsyncDrop + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> DerefMut for AsyncDropper<T>
where
    T: AsyncDrop + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

#[cfg(all(not(feature = "tokio"), not(feature = "async-std")))]
impl<T: AsyncDrop + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "either 'async-std' or 'tokio' features must be enabled for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
impl<T: AsyncDrop + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "'async-std' and 'tokio' features cannot both be specified for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "tokio", not(feature = "async-std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<T: AsyncDrop + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            // This is the current instance.
            self.dropped = true;
            // Grab timeout before mem::take since it replaces self with Default::default() which is None.
            let timeout = self.timeout;
            let mut this = std::mem::take(self);
            // This is the Default instance created anew.
            self.dropped = true;

            if let Some(time) = timeout {
                async_scoped::TokioScope::scope_and_block({
                    |s| {
                        s.spawn(tokio::time::timeout(time, async {
                            this.inner.take().expect(INFALLIBLE).async_drop().await;
                        }));
                    }
                });
            } else {
                async_scoped::TokioScope::scope_and_block({
                    |s| {
                        s.spawn(async {
                            this.inner.take().expect(INFALLIBLE).async_drop().await;
                        });
                    }
                });
            }
        }
    }
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl<T: AsyncDrop + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            // This is the current instance.
            self.dropped = true;
            // Grab timeout before mem::take since it replaces self with Default::default() which is None.
            let timeout = self.timeout;
            let mut this = std::mem::take(self);
            // This is the Default instance created anew.
            self.dropped = true;

            if let Some(time) = timeout {
                async_scoped::AsyncStdScope::scope_and_block({
                    |s| {
                        s.spawn(async_std::future::timeout(time, async {
                            this.inner.take().expect(INFALLIBLE).async_drop().await;
                        }));
                    }
                });
            } else {
                async_scoped::AsyncStdScope::scope_and_block({
                    |s| {
                        s.spawn(async {
                            this.inner.take().expect(INFALLIBLE).async_drop().await;
                        });
                    }
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "./__tests__/lib.spec.rs"]
mod tests;
