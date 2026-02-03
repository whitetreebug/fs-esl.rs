use crate::{codec::EslCodec, error::EslError, event::Event};
use futures::{SinkExt, StreamExt};
use log::{error, info, trace};
use tokio::net::TcpStream;
use uuid::Uuid;

use tokio_util::codec::{FramedRead, FramedWrite};

#[derive(Debug)]
pub enum EslEventType {
    PLAIN,
    XML,
    JSON,
}

// type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;
// type FramedWriter = SplitSink<Framed<TcpStream, EslCodec>, &[u8]>;
type FramedReader = FramedRead<tokio::io::ReadHalf<TcpStream>, EslCodec>;
type FramedWriter = FramedWrite<tokio::io::WriteHalf<TcpStream>, EslCodec>;

pub struct EslHandle {
    #[allow(unused)]
    address: String,
    user: String,
    password: String,
    reader: FramedReader,
    writer: FramedWriter,
    event_lock: bool,
    event_async_execute: bool,
    //pub background_job: HashSet<String>,
}

impl EslHandle {
    pub async fn inbound(address: &str, user: &str, password: &str) -> Result<Self, EslError> {
        let stream = TcpStream::connect(address).await?;
        let (read_half, write_half) = tokio::io::split(stream);
        let framedread = FramedRead::new(read_half, EslCodec::default());
        let framedwrite = FramedWrite::new(write_half, EslCodec::default());
        let ret = EslHandle {
            address: String::from(address),
            user: user.to_string(),
            password: password.to_string(),
            reader: framedread,
            writer: framedwrite,
            event_lock: true,
            event_async_execute: false, //background_job: HashSet::new(),
        };
        Ok(ret)
    }

    pub async fn send_recv(&mut self, cmd: &[u8]) -> Result<Event, EslError> {
        self.writer.send(cmd).await?;
        if let Some(event) = self.reader.next().await {
            return event;
        }
        Err(EslError::UnknowError)
    }
    pub async fn api(&mut self, cmd: &str, arg: &str) -> Result<Event, EslError> {
        let cmd = format!("api {} {}", cmd, arg);
        self.send_recv(cmd.as_bytes()).await
    }

    //should subscribe BACKGROUND_JOB first
    pub async fn bgapi(&mut self, cmd: &str, arg: &str) -> Option<Uuid> {
        let arg = if arg.is_empty() {
            "".to_string()
        } else {
            " ".to_string() + arg
        };
        let uuid = Uuid::new_v4();
        //self.background_job.insert(uuid.to_string());
        let cmd = format!("bgapi {}{}\nJob-UUID: {}", cmd, arg, uuid);

        match self.send_recv(cmd.as_bytes()).await {
            Ok(event) => {
                info!("{:?}", event);
                Some(uuid)
            }
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    pub async fn execute(
        &mut self,
        app: &str,
        args: &str,
        call_uuid: &str,
    ) -> Result<Event, EslError> {
        let uuid = uuid::Uuid::new_v4().to_string();
        //self.background_job.insert(uuid.to_string());

        let cmd = format!(
            "sendmsg {}\n
            execute-app-name: {}\n
            execute-app-arg: {}\n
            call-command: execute\n
            Event-UUID: {}\n
            event-lock: {}\n
            async: {}\n",
            call_uuid, app, args, uuid, self.event_lock, self.event_async_execute
        );
        info! {"{}", cmd};

        match self.send_recv(cmd.as_bytes()).await {
            Ok(event) => {
                info!("{:?}", event);
                Ok(event)
            }
            Err(e) => {
                error!("{:?}", e);
                Err(e)
            }
        }
    }

    //only plain now
    pub async fn subscribe(
        &mut self,
        event_type: EslEventType,
        events: Vec<&str>,
    ) -> Result<Event, EslError> {
        let event_type = match event_type {
            EslEventType::PLAIN => "plain",
            EslEventType::JSON => "json",
            EslEventType::XML => "xml",
        };
        let cmd = format!("event {} {}", event_type, events.join(" "));
        self.send_recv(cmd.as_bytes()).await
    }

    pub async fn auth(&mut self) -> Result<Event, EslError> {
        let cmd = match self.user.is_empty() {
            true => format!("auth {}", self.password),
            false => format!("auth {} {}", self.user, self.password),
        };
        self.send_recv(cmd.as_bytes()).await
    }

    pub async fn disconnect(&mut self) -> Result<(), EslError> {
        self.send_recv(b"exit").await?;
        Ok(())
    }

    pub async fn start_events_listen(&mut self, func: &dyn Fn(&Event)) {
        loop {
            match self.reader.next().await {
                None => {
                    trace!("framd_reader read none");
                    break;
                }
                Some(Err(e)) => {
                    error!("{:?}", e);
                    break;
                }
                Some(Ok(event)) => {
                    func(&event);

                    /*
                    if let Some(uuid) = event.get_header("Job-UUID") {
                        if self.background_job.contains(uuid) {
                            self.background_job.remove(uuid);
                        } else {
                            error!("extra uuid {:?}", uuid);
                        }
                    }
                    */
                }
            }
        }
    }
}
