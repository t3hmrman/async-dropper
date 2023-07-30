use std::{
    result::Result,
    time::Duration,
};

use async_dropper::derive::AsyncDrop;
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped
///
/// Objects that are dropped *must* implement [Default] and [PartialEq]
/// (so make members optional, hide them behind Rc/Arc as necessary)
#[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
struct AsyncThing(String);

/// Implementation of [AsyncDrop] that specifies the actual behavior
#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        // Wait 2 seconds then "succeed"
        eprintln!("async dropping [{}]!", self.0);
        async_std::task::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{}]!", self.0);
        Ok(())
    }

    fn reset(&mut self) {
        self.0 = String::default();
    }

    fn drop_timeout(&self) -> Duration {
        Duration::from_secs(5) // extended from default 3 seconds
    }

    // NOTE: below was not implemented since we want the default of DropFailAction::Contineue
    // fn drop_fail_action(&self) -> DropFailAction;
}

#[async_std::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(AsyncThing(String::from("test")));
    Ok(())
}
