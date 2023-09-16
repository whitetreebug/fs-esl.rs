use fs_esl::{
    error::EslError,
    esl::{EslEventType, EslHandle},
    event::Event,
};
use log::{error, info, trace};
use std::time::SystemTime;

fn func(event: Event) {
    match event.get_header("Event-Name") {
        Some("CHANNEL_ANSWER") => {}
        Some(_) => {
            trace!("{:?}", event);
        }
        None => {
            trace!("{:?}", event);
        }
    }
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
    let mut handle = EslHandle::inbound("127.0.0.1:8021", "", "ClueCon").await?;
    handle.auth().await?;
    let events = vec![
        "BACKGROUND_JOB",
        "CHANNEL_CREATE",
        "CHANNEL_ANSWER",
        "CHANNEL_PROGRESS",
        "CHANNEL_PROGRESS_MEDIA",
        "CHANNEL_BRIDGE",
        "CHANNEL_EXECUTE_COMPLETE",
        "CHANNEL_HANGUP_COMPLETE",
    ];
    //let events = vec!["ALL"];
    handle.subscribe(EslEventType::PLAIN, events).await.unwrap();

    EslHandle::start_events_listen(&mut handle.reader, &func).await;
    Ok(())
}
