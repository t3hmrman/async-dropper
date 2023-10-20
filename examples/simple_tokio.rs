use std::{
    result::Result,
    time::Duration,
};

use async_dropper_simple::{AsyncDrop, AsyncDropper};
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped (which must be wrapped in AsyncDropper)
#[derive(Default)]
struct AsyncThing(String);

#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) {
        eprintln!("async dropping [{}]!", self.0);
        tokio::time::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{}]!", self.0);
    }
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(AsyncDropper::new(AsyncThing(String::from("test"))));
    Ok(())
}
