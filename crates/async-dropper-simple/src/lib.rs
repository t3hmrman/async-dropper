#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
//! The code in this file was shamelessly stolen from
//! https://stackoverflow.com/questions/71541765/rust-async-drop

use std::time::Duration;

#[async_trait::async_trait]
pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

#[derive(Default)]
#[allow(dead_code)]
pub struct AsyncDropper<T: AsyncDrop + Default + Send + 'static> {
    dropped: bool,
    timeout: Option<Duration>,
    inner: T,
}

impl<T: AsyncDrop + Default + Send + 'static> AsyncDropper<T> {
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
}

#[cfg(all(not(feature = "tokio"), not(feature = "async-std")))]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "either 'async-std' or 'tokio' features must be enabled for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        compile_error!(
            "'async-std' and 'tokio' features cannot both be specified for the async-dropper crate"
        )
    }
}

#[cfg(all(feature = "tokio", not(feature = "async-std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
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
            let task = tokio::spawn(async move {
                this.inner.async_drop().await;
            });

            tokio::task::block_in_place(|| match timeout {
                Some(d) => {
                    let _ = futures::executor::block_on(tokio::time::timeout(d, task));
                }
                None => {
                    let _ = futures::executor::block_on(task);
                }
            });
        }
    }
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
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
            let task = async_std::task::spawn(async move {
                this.inner.async_drop().await;
            });

            match timeout {
                Some(d) => {
                    let _ = futures::executor::block_on(async_std::future::timeout(d, task));
                }
                None => {
                    let _ = futures::executor::block_on(task);
                }
            };
        }
    }
}
