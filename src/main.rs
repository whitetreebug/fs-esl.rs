use fs_esl::{
    error::EslError,
    esl::{EslEventType, EslHandle},
};
use std::{env::set_var, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), EslError> {
    set_var("RUST_LOG", "trace");
    env_logger::init();

    if let Ok(mut handle) = EslHandle::connect("127.0.0.1:8021", "", "ClueCon").await {
        handle.auth().await.expect("auth error");

        handle.send("log info".to_string()).await?;
        //handle.recv(Duration::from_secs(0)).await?;
        //handle.bgapi("", "", false).await?;
        //handle.events(EslEventType::ESLEVENTTYPEJSON, "ALL").await?;
        loop {
            sleep(Duration::from_secs(2)).await;
        }
    }

    Ok(())
}
