use fs_esl::{
    codec::EslCodec,
    error::EslError,
    esl::{EslEventType, EslHandle},
    event::Event,
};
use futures::{stream::SplitStream, StreamExt};
use log::{error, trace};
use std::{
    collections::VecDeque,
    env::set_var,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpStream, sync::oneshot::Sender};
use tokio_util::codec::Framed;
type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;

async fn events_listen(
    mut framed_reader: FramedReader,
    inner_commands: Arc<Mutex<VecDeque<Sender<Event>>>>,
) {
    loop {
        match framed_reader.next().await {
            None => {
                trace!("framd_reader read none");
                break;
            }
            Some(Err(e)) => {
                error!("{:?}", e);
                break;
            }
            Some(Ok(event)) => {
                if let Some(tx) = inner_commands.lock().unwrap().pop_front() {
                    if let Err(e) = tx.send(event) {
                        error!("{:?}", e);
                    }
                }
            }
        }
    }
}

async fn operator(handle: &mut EslHandle) {
    handle.auth().await.unwrap();
    handle.events(EslEventType::PLAIN, "ALL").await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), EslError> {
    set_var("RUST_LOG", "trace");
    env_logger::init();

    let stream = TcpStream::connect("127.0.0.1:8021").await?;
    let framed = Framed::new(stream, EslCodec::new());
    let (framed_writer, framed_reader) = framed.split::<String>();

    let mut handle = EslHandle::inbound(framed_writer, "", "ClueCon")
        .await
        .unwrap();

    let inner_commands = handle.command.clone();

    tokio::join!(
        events_listen(framed_reader, inner_commands),
        operator(&mut handle)
    );
    Ok(())
}