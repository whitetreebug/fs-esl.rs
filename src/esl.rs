use crate::{codec::EslCodec, error::EslError, event::Event};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info, trace};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpStream, time};
use uuid::Uuid;

use tokio::sync::oneshot::{self, Sender};
use tokio_util::codec::{Framed, FramedRead, FramedWrite};

#[derive(Debug)]
pub enum EslEventType {
    PLAIN,
    XML,
    JSON,
}

type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;
type FramedWriter = SplitSink<Framed<TcpStream, EslCodec>, String>;

pub struct EslHandle {
    socket: String,
    user: String,
    password: String,
    pub reader: FramedReader,
    pub writer: FramedWriter,
    pub background_job: HashSet<String>,
    //   pub command: Arc<Mutex<VecDeque<Sender<Event>>>>,
    //  pub background_job: Arc<Mutex<HashMap<String, Sender<Event>>>>,
}

impl EslHandle {
    pub async fn inbound(socket: &str, user: &str, password: &str) -> Result<Self, EslError> {
        let stream = TcpStream::connect(socket).await?;
        let framed = Framed::new(stream, EslCodec::new());
        let (framedwrite, framedread) = framed.split();
        let ret = EslHandle {
            socket: String::from(socket),
            user: user.to_string(),
            password: password.to_string(),
            reader: framedread,
            writer: framedwrite,
            background_job: HashSet::new(),
        };

        Ok(ret)
    }

    pub async fn send_recv(&mut self, cmd: String) -> Result<Event, EslError> {
        self.writer.send(cmd).await?;
        while let Some(event) = self.reader.next().await {
            return event;
        }
        Err(EslError::UnknowError)
    }
    pub async fn api(&mut self, cmd: &str, arg: &str) -> Result<Event, EslError> {
        let cmd = format!("api {} {}", cmd, arg);
        self.send_recv(cmd).await
    }

    //should subscribe BACKGROUND_JOB first
    pub async fn bgapi(&mut self, cmd: &str, arg: &str) -> Result<Event, EslError> {
        let arg = if arg.is_empty() {
            "".to_string()
        } else {
            " ".to_string() + arg
        };
        let uuid = Uuid::new_v4();
        self.background_job.insert(uuid.to_string());
        let command = format!("bgapi {}{}\nJob-UUID: {}", cmd, arg, uuid);

        match self.send_recv(command).await {
            Ok(event) => {
                info!("{:?}", event);
                return Ok(event);
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(e);
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
        self.background_job.insert(uuid.to_string());
        let command  = format!("sendmsg {}\nexecute-app-name: {}\nexecute-app-arg: {}\ncall-command: execute\nEvent-UUID: {}",call_uuid,app,args,uuid);
        info! {"{}", command};

        match self.send_recv(command).await {
            Ok(event) => {
                info!("{:?}", event);
                return Ok(event);
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(e);
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
        self.send_recv(format!("event {} {}", event_type, events.join(" ")))
            .await
    }

    pub async fn auth(&mut self) -> Result<Event, EslError> {
        let cmd;
        if self.user.is_empty() {
            cmd = format!("auth {}", self.password);
        } else {
            cmd = format!("auth {} {}", self.user, self.password);
        }
        self.send_recv(cmd).await
    }

    pub async fn disconnect(&mut self) -> Result<(), EslError> {
        self.send_recv("exit".to_string()).await?;
        Ok(())
    }

    pub async fn start_events_listen(reader: &mut FramedReader, func: &dyn Fn(Event)) {
        loop {
            match reader.next().await {
                None => {
                    trace!("framd_reader read none");
                    break;
                }
                Some(Err(e)) => {
                    error!("{:?}", e);
                    break;
                }
                Some(Ok(event)) => {
                    func(event);

                    // if let Some(uuid) = event.get_header("Job-UUID") {
                    //     if self.background_job.contains(uuid) {
                    //         self.background_job.remove(uuid);
                    //     } else {
                    //         error!("extra uuid {:?}", uuid);
                    //     }
                    // }
                }
            }
        }
    }
}
