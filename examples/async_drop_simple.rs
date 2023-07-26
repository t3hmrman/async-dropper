use std::{
    result::Result,
};

use async_dropper::simple::AsyncDropper;
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped (which must be wrapped in AsyncDropper) 
struct AsyncThing(String);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    { 
        let _example_obj = AsyncDropper::new(AsyncThing(String::from("test")));
        eprintln!("here comes the (async) drop");
        // drop will be triggered here, and it will take *however long it takes*
        // you could also call `drop(_example_obj)`
    }

    Ok(())
    // Another drop happens after the function, but that one will be a no-op
}
