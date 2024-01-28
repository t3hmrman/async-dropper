use super::*;
use macros_internal::test_async_runtime;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct TestDropper<'a> {
    callback: Option<Box<dyn FnOnce() + Send + 'a>>,
    value: usize,
    timeout: Duration,
}

impl<'a> TestDropper<'a> {
    fn new(callback: impl FnOnce() + Send + 'a, timeout: Option<Duration>) -> Self {
        Self {
            callback: Some(Box::new(callback)),
            value: 0,
            timeout: timeout.unwrap_or_else(|| Duration::from_millis(100)),
        }
    }
}

#[async_trait::async_trait]
impl AsyncDrop for TestDropper<'static>
where
    Self: Send,
{
    async fn async_drop(&mut self) {
        if cfg!(feature = "tokio") {
            tokio::time::sleep(self.timeout).await;
        } else if cfg!(feature = "async-std") {
            async_std::task::sleep(self.timeout).await;
        }

        self.value += 1;

        if let Some(x) = self.callback.take() {
            x();
        }
    }
}

#[test_async_runtime]
async fn test_dropper_waiting_for_drop() {
    let counter = Arc::new(AtomicUsize::default());
    let inner = TestDropper::new(
        {
            let counter = counter.clone();

            move || {
                counter.fetch_add(1, Ordering::AcqRel);
            }
        },
        None,
    );
    let instance = AsyncDropper::new(inner);
    assert_eq!(counter.load(Ordering::Acquire), 0);
    drop(instance);
    assert_eq!(counter.load(Ordering::Acquire), 1);
}

#[test_async_runtime]
async fn test_dropper_timeout() {
    let counter = Arc::new(AtomicUsize::default());
    let inner = TestDropper::new(
        {
            let counter = counter.clone();

            move || {
                counter.fetch_add(1, Ordering::AcqRel);
            }
        },
        Some(Duration::from_secs(100)),
    );
    let instance = AsyncDropper::with_timeout(Duration::from_nanos(1), inner);
    assert_eq!(counter.load(Ordering::Acquire), 0);
    drop(instance);
    assert_eq!(counter.load(Ordering::Acquire), 0);
}

#[test_async_runtime]
async fn test_derefs() {
    let inner = TestDropper::new(|| {}, None);
    let mut instance = AsyncDropper::new(inner);

    {
        let inn = &*instance;
        assert_eq!(inn.value, 0);
    }

    {
        let inn = instance.inner();
        assert_eq!(inn.value, 0);
    }

    {
        let inn = &mut *instance;
        inn.value += 1;
        assert_eq!(inn.value, 1);
    }

    {
        let inn = instance.inner_mut();
        inn.value += 1;
        assert_eq!(inn.value, 2);
    }
}
