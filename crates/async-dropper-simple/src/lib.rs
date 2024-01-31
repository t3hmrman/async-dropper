#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
//! The code in this file was shamelessly stolen from
//! https://stackoverflow.com/questions/71541765/rust-async-drop

use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[async_trait::async_trait]
pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

#[derive(Default)]
#[allow(dead_code)]
pub struct AsyncDropper<T: AsyncDrop + Default + Send> {
    dropped: bool,
    timeout: Option<Duration>,
    inner: T,
}

impl<T: AsyncDrop + Default + Send> AsyncDropper<T> {
    pub fn new(inner: T) -> Self {
        Self {
            dropped: false,
            timeout: None,
            inner,
        }
    }

    pub fn with_timeout(timeout: Duration, inner: T) -> Self {
        Self {
            dropped: false,
            timeout: Some(timeout),
            inner,
        }
    }

    /// Get a reference to the inner data
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a mutable refrence to inner data
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Deref for AsyncDropper<T>
where
    T: AsyncDrop + Send + Default,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.inner()
    }
}

impl<T> DerefMut for AsyncDropper<T>
where
    T: AsyncDrop + Send + Default,
{
    fn deref_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

#[cfg(all(not(feature = "tokio"), not(feature = "async-std")))]
impl<T: AsyncDrop + Default + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "either 'async-std' or 'tokio' features must be enabled for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
impl<T: AsyncDrop + Default + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "'async-std' and 'tokio' features cannot both be specified for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "tokio", not(feature = "async-std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<T: AsyncDrop + Default + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            use async_scoped::TokioScope;

            // Set the original instance to be dropped
            self.dropped = true;

            // Save the timeout on the original instance
            let timeout = self.timeout;

            // Swap out the current instance with default
            // (i.e. `this` is now original instance, and `self` is a default instance)
            let mut this = std::mem::take(self);

            // Set the default instance to note that it's dropped
            self.dropped = true;

            // Create task
            match timeout {
                // If a timeout was specified, use it when performing async_drop
                Some(d) => {
                    TokioScope::scope_and_block(|s| {
                        s.spawn(tokio::time::timeout(d, async move {
                            this.inner.async_drop().await;
                        }))
                    });
                }
                // If no timeout was specified, perform async_drop() indefinitely
                None => {
                    TokioScope::scope_and_block(|s| {
                        s.spawn(async move {
                            this.inner.async_drop().await;
                        })
                    });
                }
            }
        }
    }
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl<T: AsyncDrop + Default + Send> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            use async_scoped::AsyncStdScope;

            // Set the original instance to be dropped
            self.dropped = true;

            // Save the timeout on the original instance
            let timeout = self.timeout;

            // Swap out the current instance with default
            // (i.e. `this` is now original instance, and `self` is a default instance)
            let mut this = std::mem::take(self);

            // Set the default instance to note that it's dropped
            self.dropped = true;

            match timeout {
                // If a timeout was specified, use it when performing async_drop
                Some(d) => {
                    AsyncStdScope::scope_and_block(|s| {
                        s.spawn(async_std::future::timeout(d, async move {
                            this.inner.async_drop().await;
                        }))
                    });
                }
                // If no timeout was specified, perform async_drop() indefinitely
                None => {
                    AsyncStdScope::scope_and_block(|s| {
                        s.spawn(async move {
                            this.inner.async_drop().await;
                        })
                    });
                }
            }
        }
    }
}
