//! Implementation of simple AsyncDropper that does not require `Default`
//!
//! This implementation might be preferrable for people who cannot reasonably
//! implement `Default` for their struct, but have easily accessible `Eq`,`PartialEq`,
//! `Hash`, and/or `Clone` instances.
#![cfg(feature = "no-default-bound")]

use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::AsyncDrop;

/// Wrapper struct that enables `async_drop()` behavior.
///
/// This version does not require `Default`, via the user of an inner `Option<T>`
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[allow(dead_code)]
pub struct AsyncDropper<T: AsyncDrop + Send + 'static> {
    dropped: bool,
    timeout: Option<Duration>,
    inner: Option<T>,
}

impl<T: AsyncDrop + Send + 'static> AsyncDropper<T> {
    /// Create an `AsyncDropper<T>` without a timeout
    pub fn new(inner: T) -> Self {
        Self {
            dropped: false,
            timeout: None,
            inner: Some(inner),
        }
    }

    /// Create an `AsyncDropper<T>` with a given timeout
    pub fn with_timeout(timeout: Duration, inner: T) -> Self {
        Self {
            dropped: false,
            timeout: Some(timeout),
            inner: Some(inner),
        }
    }

    /// Get a reference to the inner data
    pub fn inner(&self) -> &T {
        self.inner
            .as_ref()
            .expect("failed to retreive inner content")
    }

    /// Get a mutable refrence to inner data
    pub fn inner_mut(&mut self) -> &mut T {
        self.inner
            .as_mut()
            .expect("failed to retrieve inner content")
    }
}

/// Choosing to *not* create the default T instance
/// means that you must rely on `new()` or `with_timeout()` to create
/// new `AsyncDropper` instances that have an `inner` specified
impl<T: AsyncDrop + Send> Default for AsyncDropper<T> {
    fn default() -> Self {
        Self {
            dropped: false,
            timeout: None,
            inner: None,
        }
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
impl<T: AsyncDrop + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "either 'async-std' or 'tokio' features must be enabled for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
impl<T: AsyncDrop + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "'async-std' and 'tokio' features cannot both be specified for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "tokio", not(feature = "async-std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<T: AsyncDrop + Send + 'static> Drop for AsyncDropper<T> {
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
                            this.inner
                                .take()
                                .expect(
                                    "unexpectedly failed to take ownership AsyncDropper inner data",
                                )
                                .async_drop()
                                .await;
                        }))
                    });
                }
                // If no timeout was specified, perform async_drop() indefinitely
                None => {
                    TokioScope::scope_and_block(|s| {
                        s.spawn(async move {
                            this.inner
                                .take()
                                .expect(
                                    "unexpectedly failed to take ownership AsyncDropper inner data",
                                )
                                .async_drop()
                                .await;
                        })
                    });
                }
            }
        }
    }
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl<T: AsyncDrop + Send + 'static> Drop for AsyncDropper<T> {
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
                            this.inner
                                .take()
                                .expect(
                                    "unexpectedly failed to take ownership AsyncDropper inner data",
                                )
                                .async_drop()
                                .await;
                        }))
                    });
                }
                // If no timeout was specified, perform async_drop() indefinitely
                None => {
                    AsyncStdScope::scope_and_block(|s| {
                        s.spawn(async move {
                            this.inner
                                .take()
                                .expect(
                                    "unexpectedly failed to take ownership AsyncDropper inner data",
                                )
                                .async_drop()
                                .await;
                        })
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;

    use crate::{AsyncDrop, AsyncDropper};

    /// Testing struct which contains an arc to an atomic counter
    /// so that we can tell how far drop gets, if necessary
    struct Test {
        // This counter is used as an indicator of how far async_drop() gets
        // - 0 means async_drop() never ran
        // - 1 means async_drop() started but did not complete
        // - 2 means async_drop() completed
        counter: Arc<AtomicU32>,
    }

    #[async_trait]
    impl AsyncDrop for Test {
        async fn async_drop(&mut self) {
            self.counter.store(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.counter.store(2, Ordering::SeqCst);
        }
    }

    /// Ensure that non-`Default`-bounded dropper works with tokio
    #[cfg(feature = "tokio")]
    #[tokio::test(flavor = "multi_thread")]
    async fn tokio_works() {
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));

        // Create and perform drop
        let wrapped_t = AsyncDropper::new(Test {
            counter: counter.clone(),
        });
        drop(wrapped_t);

        assert!(
            start.elapsed() > Duration::from_millis(500),
            "two seconds have passed since drop"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            2,
            "async_drop() ran to completion"
        );
    }

    // TODO: this test is broken *because* of the timeout bug
    // see: https://github.com/t3hmrman/async-dropper/pull/17
    /// Ensure that non-`Default`-bounded dropper works with tokio with a timeout
    #[cfg(feature = "tokio")]
    #[tokio::test(flavor = "multi_thread")]
    async fn tokio_works_with_timeout() {
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));
        let wrapped_t = AsyncDropper::with_timeout(
            Duration::from_millis(500),
            Test {
                counter: counter.clone(),
            },
        );
        drop(wrapped_t);
        assert!(
            start.elapsed() > Duration::from_millis(500),
            "two seconds have passed since drop"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "async_drop() did not run to completion (should have timed out)"
        );
    }

    /// Ensure that non-`Default`-bounded dropper works with async-std
    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn async_std_works() {
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));

        let wrapped_t = AsyncDropper::new(Test {
            counter: counter.clone(),
        });
        drop(wrapped_t);

        assert!(
            start.elapsed() > Duration::from_millis(500),
            "two seconds have passed since drop"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            2,
            "async_drop() ran to completion"
        );
    }

    // TODO: this test is broken *because* of the timeout bug
    // see: https://github.com/t3hmrman/async-dropper/pull/17
    /// Ensure that non-`Default`-bounded dropper works with async-std with a timeout
    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn async_std_works_with_timeout() {
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));
        let wrapped_t = AsyncDropper::with_timeout(
            Duration::from_millis(500),
            Test {
                counter: counter.clone(),
            },
        );
        drop(wrapped_t);
        assert!(
            start.elapsed() > Duration::from_millis(500),
            "two seconds have passed since drop"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "async_drop() did not run to completion (should have timed out)"
        );
    }
}
