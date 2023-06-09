use fs_esl::{
    codec::EslCodec,
    error::EslError,
    esl::{EslEventType, EslHandle},
    event::Event,
};
use futures::{stream::SplitStream, StreamExt};
use log::{error, info, trace};
use std::{
    collections::{HashMap, VecDeque},
    env::set_var,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use tokio::{net::TcpStream, sync::oneshot::Sender};
use tokio_util::codec::Framed;
type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;

async fn events_listen(
    mut framed_reader: FramedReader,
    inner_commands: Arc<Mutex<VecDeque<Sender<Event>>>>,
    background_job: Arc<Mutex<HashMap<String, Sender<Event>>>>,
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
                } else {
                    if let Some(uuid) = event.get_header("Job-UUID") {
                        if let Some(tx) = background_job.lock().unwrap().remove(uuid) {
                            if let Err(e) = tx.send(event) {
                                error!("{:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn operator(handle: &mut EslHandle) {
    originate(handle).await;
    /*
    handle.auth().await.unwrap();
    //let events = vec!["CHANNEL_EXECUTE_COMPLETE"];
    //let events = vec!["BACKGROUND_JOB"];
    let events = vec!["ALL"];
    handle.events(EslEventType::JSON, events).await.unwrap();
    let event = handle.bgapi("reloadxml", "").await.unwrap();
    info!("{:?}", event);
    */
}

async fn originate(handle: &mut EslHandle) {
    //should subscribe BACKGROUND_JOB first
    handle.auth().await.unwrap();
    let events = vec!["BACKGROUND_JOB", "CHANNEL_CALLSTATE"];
    handle.events(EslEventType::PLAIN, events).await.unwrap();

    handle
        .bgapi(
            "originate {absolute_codec_string=PCMA,effective_caller_id_number=1001}user/1001",
            "&bridge(user/1000)",
        )
        .await
        .unwrap();
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), EslError> {
    setup_logger().unwrap();

    let stream = TcpStream::connect("127.0.0.1:8021").await?;
    let framed = Framed::new(stream, EslCodec::new());
    let (framed_writer, framed_reader) = framed.split::<String>();

    let mut handle = EslHandle::inbound(framed_writer, "", "ClueCon")
        .await
        .unwrap();

    let inner_commands = handle.command.clone();
    let backgroud_job = handle.background_job.clone();

    tokio::join!(
        events_listen(framed_reader, inner_commands, backgroud_job),
        operator(&mut handle)
    );
    Ok(())
}
