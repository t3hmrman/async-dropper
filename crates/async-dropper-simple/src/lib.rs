///! The code in this file was shamelessly stolen from
///! https://stackoverflow.com/questions/71541765/rust-async-drop
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
        panic!("either 'async-std' or 'tokio' features must be enabled for the async-dropper crate")
    }
}

#[cfg(feature = "tokio")]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            let mut this = AsyncDropper::default();
            std::mem::swap(&mut this, self);
            this.dropped = true;

            // Create task
            let timeout = self.timeout.clone();
            let task = tokio::spawn(async move {
                this.inner.async_drop().await;
            });

            match timeout {
                Some(d) => {
                    let _ = futures::executor::block_on(tokio::time::timeout(d, task));
                },
                None => {
                    let _ = futures::executor::block_on(task);
                },
            };
        }
    }
}

#[cfg(feature = "async-std")]
impl<T: AsyncDrop + Default + Send + 'static> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        if !self.dropped {
            let mut this = AsyncDropper::default();
            std::mem::swap(&mut this, self);
            this.dropped = true;

            // Create task
            let timeout = self.timeout.clone();
            let task = async_std::task::spawn(async move {
                this.inner.async_drop().await;
            });

            match timeout {
                Some(d) => {
                    let _ = futures::executor::block_on(async_std::future::timeout(d, task));
                },
                None => {
                    let _ = futures::executor::block_on(task);
                },
            };

        }
    }
}
