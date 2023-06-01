use fs_esl::esl::EslHandle;
use std::env::set_var;

#[tokio::main]
async fn main() {
    set_var("RUST_LOG", "trace");
    env_logger::init();
    //let mut handle = EslHandle::new("",);
    EslHandle::connect("", "ClueCon", "127.0.0.1:8021")
        .await
        .unwrap();
}
