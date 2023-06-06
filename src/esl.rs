use crate::{codec::EslCodec, error::EslError, event::Event};
use futures::{stream::SplitSink, SinkExt};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpStream, time};
use uuid::Uuid;

use tokio::sync::oneshot::{self, Sender};
use tokio_util::codec::Framed;

#[derive(Debug)]
pub enum EslEventType {
    PLAIN,
    XML,
    JSON,
}

type FramedWriter = SplitSink<Framed<TcpStream, EslCodec>, String>;
//type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;

pub struct EslHandle {
    user: String,
    password: String,
    pub command: Arc<Mutex<VecDeque<Sender<Event>>>>,
    pub background_job: Arc<Mutex<HashMap<String, Sender<Event>>>>,
    framed_writer: FramedWriter,
}

pub struct EslMsg {}

impl EslHandle {
    pub async fn inbound(
        framed_writer: FramedWriter,
        user: &str,
        password: &str,
    ) -> Result<Self, EslError> {
        let handle = EslHandle {
            user: user.to_string(),
            password: password.to_string(),
            command: Arc::new(Mutex::new(VecDeque::new())),
            background_job: Arc::new(Mutex::new(HashMap::new())),
            framed_writer,
        };

        Ok(handle)
    }

    /*
    pub async fn connect_timeout(
        &mut self,
        addr: impl ToSocketAddrs,
        time_out_sec: u64,
    ) -> Result<&EslHandle, EslError> {
        let stream = match time::timeout(
            Duration::from_secs(time_out_sec),
            TcpStream::connect(addr),
        )
        .await
        {
            Ok(stream) => match stream {
                Ok(stream) => Some(stream),
                Err(_) => None,
            },
            Err(e) => {
                format!("timeout while connecting to server: {}", e.to_string()); // to do log
                None
            }
        };

        Ok(self)
    }
    */

    pub async fn send(&mut self, cmd: String) -> Result<(), EslError> {
        match self.framed_writer.feed(cmd).await {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    pub async fn recv(&mut self, duration: Duration) -> Result<Event, EslError> {
        self.framed_writer.flush().await?;
        let (tx, rx) = oneshot::channel();
        self.command.lock().unwrap().push_back(tx);

        if duration.is_zero() {
            match rx.await {
                Ok(event) => Ok(event),
                Err(e) => return Err(EslError::RecvError(e)),
            }
        } else {
            let res = time::timeout(duration, async { rx.await }).await;
            match res {
                Ok(res) => match res {
                    Ok(res) => return Ok(res),
                    Err(e) => return Err(EslError::RecvError(e)),
                },
                Err(e) => return Err(EslError::ElapsedError(e)),
            }
        }
    }

    pub async fn send_recv(&mut self, cmd: String) -> Result<Event, EslError> {
        self.framed_writer.send(cmd).await?;
        let (tx, rx) = oneshot::channel();
        self.command.lock().unwrap().push_back(tx);

        match rx.await {
            Ok(event) => return Ok(event),
            Err(e) => return Err(EslError::RecvError(e)),
        }
    }

    pub async fn api(&mut self, cmd: String, arg: String) -> Result<Event, EslError> {
        let cmd = format!("{} {}", cmd, arg);
        self.send_recv(cmd).await
    }

    pub async fn bgapi(&mut self, cmd: &str, arg: &str) -> Result<Event, EslError> {
        let arg = if arg.is_empty() {
            "".to_string()
        } else {
            " ".to_string() + arg
        };
        let (tx, rx) = oneshot::channel();
        let uuid = Uuid::new_v4();
        self.background_job.lock().unwrap().insert(uuid.to_string(), tx);
        let command = format!("bgapi {}{}\nJob-UUID: {}", cmd, arg, uuid);
        self.send_recv(command).await?;

        match rx.await {
            Ok(event) => return Ok(event),
            Err(e) => return Err(EslError::RecvError(e)),
        }
    }

    //only plain now
    pub async fn events(
        &mut self,
        event_type: EslEventType,
        channel_event: &str,
    ) -> Result<Event, EslError> {
        let event_type = match event_type {
            EslEventType::PLAIN => "plain",
            EslEventType::JSON => "json",
            EslEventType::XML => "xml",
        };
        self.send_recv(format!("event {} {}", event_type, channel_event))
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
}
