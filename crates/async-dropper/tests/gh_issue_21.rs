use std::sync::atomic::AtomicBool;
use std::sync::Weak;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use async_dropper::{AsyncDrop, AsyncDropError};

/// Ensure that async-dropper works with atomic structures and weak refs
///
/// see: https://github.com/t3hmrman/async-dropper/issues/21
#[tokio::test]
async fn gh_issue_21() -> Result<()> {
    #[derive(Default)]
    pub struct State(String);

    #[derive(Default, PartialEq, Eq, AsyncDrop)]
    pub struct Supervisor {
        pub socket_bot_id: Uuid,
        pub room_id: String,
        pub private_room: AtomicBool,
        pub state: Weak<State>,
    }

    #[async_trait]
    impl AsyncDrop for Supervisor {
        async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(())
        }
    }

    let mut supervisor = Supervisor::default();
    supervisor.room_id = "test".into();
        
    drop(supervisor);

    Ok(())
}
