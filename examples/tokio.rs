use std::{
    result::Result,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_dropper::AsyncDrop;

#[derive(Debug, Error)]
enum ExampleError {
    #[error("not done counting yet")]
    NotDoneCountingError,

    #[error("mutex encounted a poison error")]
    MutexPoisonError,
}

/// This object will be async-dropped
///
/// Objects that are dropped *must* implement Default and PartialEq
/// (so make members optional, hide them behind Rc/Arc as necessary)
#[derive(Default, PartialEq, AsyncDrop)]
struct ExampleObj(&str);

/// Implementation of AsyncDrop that specifies the actual behavior
#[async_trait]
impl AsyncDrop for ExampleObject {
    async fn drop(&self) -> Result<(), AsyncDropFailure> {
        // Wait 2 seconds then "succeed"
        tokio::sleep(Duration::from_secs(2)).await;
        eprintln!("dropping [{}]!", self.0);
        Ok(())
    }

    fn drop_timeout() -> Duration {
        Duration::from_secs(5) // extended from default 3 seconds
    }

    // NOTE: below was not implemented since we want the default of DropFailAction::Contineue
    // fn drop_fail_action() -> DropFailAction; 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_obj = ExampleObj("test");
    eprintln!("here comes the (async) drop");
    drop(example_obj);

    Ok(())
    // Another drop happens after the function, but that one will be a no-op
}
