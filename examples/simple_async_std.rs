use std::result::Result;
use std::time::Duration;

use async_dropper::{AsyncDrop, AsyncDropper};
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped (which must be wrapped in AsyncDropper)
#[derive(Default)]
struct AsyncThing(String);

#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) {
        eprintln!("async dropping [{}]!", self.0);
        async_std::task::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{}]!", self.0);
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(AsyncDropper::new(AsyncThing(String::from("test"))));
    Ok(())
}
