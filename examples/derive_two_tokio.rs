use std::{result::Result, time::Duration};

use async_dropper::{AsyncDrop, AsyncDropError};
use async_trait::async_trait;

#[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
struct AsyncThing {
    value: String,
}

#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        eprintln!("async dropping thing one [{:?}]!", self);
        tokio::time::sleep(Duration::from_millis(500)).await;
        eprintln!("dropped thing one [{:?}]!", self);
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
struct AsyncThingTwo {
    value: String,
}

#[async_trait]
impl AsyncDrop for AsyncThingTwo {
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        eprintln!("async dropping thing two [{:?}]!", self);
        tokio::time::sleep(Duration::from_millis(500)).await;
        eprintln!("dropped thing two [{:?}]!", self);
        Ok(())
    }
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(AsyncThing {
        value: String::from("one"),
    });
    drop(AsyncThingTwo {
        value: String::from("two"),
    });
    Ok(())
}
