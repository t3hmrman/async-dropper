use std::result::Result;
use std::time::Duration;

use async_dropper::{AsyncDrop, AsyncDropError};
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped
///
/// Objects that are dropped *must* implement [Default] and [PartialEq]
/// (so make members optional, hide them behind Rc/Arc as necessary)
#[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
struct AsyncThing {
    value: String,
}

// async_dropper also works with tuple structs!
//
// #[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
// struct AsyncTupleThing(String);

/// Implementation of [AsyncDrop] that specifies the actual behavior
#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        // Wait 2 seconds then "succeed"
        eprintln!("async dropping [{:?}]!", self);
        tokio::time::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{:?}]!", self);
        Ok(())
    }

    fn drop_timeout(&self) -> Duration {
        Duration::from_secs(5) // extended from default 3 seconds, as an example
    }

    // NOTE: the method below is automatically derived for you, but you can override it
    // make sure that the object is equal to T::default() by the end, otherwise it will panic!
    // fn reset(&mut self) {
    //     self.value = String::default();
    // }

    // NOTE: below was not implemented since we want the default of DropFailAction::Continue
    // fn drop_fail_action(&self) -> DropFailAction;
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(AsyncThing { value: String::from("test")});
    Ok(())
}
