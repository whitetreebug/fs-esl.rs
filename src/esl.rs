use crate::{codec::EslCodec, error::EslError, event::Event};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, trace};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    time,
};
use uuid::Uuid;

use tokio::sync::oneshot::{self, Sender};
use tokio_util::codec::Framed;

#[derive(Debug)]
pub enum EslEventType {
    ESLEVENTTYPEPLAIN,
    ESLEVENTTYPEXML,
    ESLEVENTTYPEJSON,
}

type FramedWriter = SplitSink<Framed<TcpStream, EslCodec>, String>;
type FramedReader = SplitStream<Framed<TcpStream, EslCodec>>;

pub struct EslHandle {
    user: String,
    password: String,
    frame_writer: FramedWriter,
    command: Arc<Mutex<VecDeque<Sender<Event>>>>,
    // transport_rx: Arc<Mutex<FramedRead<ReadHalf<TcpStream>, EslCodec>>>,
    // transport_tx: Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, EslCodec>>>,
    // channel_rx: Receiver<String>,
}

pub struct EslMsg {}

impl EslHandle {
    pub async fn connect(
        addr: impl ToSocketAddrs,
        user: &str,
        password: &str,
    ) -> Result<Self, EslError> {
        let stream: TcpStream = TcpStream::connect(addr).await?;
        //let (tx, mut rx) = mpsc::channel(32);
        let framed = Framed::new(stream, EslCodec::new());
        let (frame_writer, mut frame_reader) = framed.split::<String>();
        let handle = EslHandle {
            user: user.to_string(),
            password: password.to_string(),
            frame_writer,
            command: Arc::new(Mutex::new(VecDeque::new())),
        };
        let inner_commands = Arc::clone(&handle.command);

        tokio::spawn(async move {
            loop {
                match frame_reader.next().await {
                    None => {
                        trace!("peer closed");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("read peer error: {}", e);
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
        });

        Ok(handle)
    }

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

    pub async fn send(&mut self, cmd: String) -> Result<(), EslError> {
        match self.frame_writer.feed(cmd).await {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    pub async fn recv(&mut self, duration: Duration) -> Result<Event, EslError> {
        self.frame_writer.flush().await?;
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
        self.frame_writer.send(cmd).await?;
        let (tx, rx) = oneshot::channel();
        self.command.lock().unwrap().push_back(tx);

        match rx.await {
            Ok(event) => return Ok(event),
            Err(e) => return Err(EslError::RecvError(e)),
        }
    }

    pub async fn api(&mut self, cmd: String, arg: String) -> Result<Event, EslError> {
        let cmd = format!("{} {}\n\n", cmd, arg);
        self.send_recv(cmd).await
    }

    pub async fn bgapi(&mut self, cmd: &str, arg: &str, job_uuid: bool) -> Result<Event, EslError> {
        let arg = if arg.is_empty() {
            "".to_string()
        } else {
            " ".to_string() + arg
        };

        let command;
        if job_uuid {
            command = format!("bgapi {}{}\nJob-UUID: {}", cmd, arg, Uuid::new_v4());
        } else {
            command = format!("bgapi {}{}", cmd, arg);
        }

        self.send_recv(command).await
    }

    pub async fn events(
        &mut self,
        event_type: EslEventType,
        channel_event: &str,
    ) -> Result<Event, EslError> {
        let event_type = match event_type {
            EslEventType::ESLEVENTTYPEPLAIN => "plain",
            EslEventType::ESLEVENTTYPEJSON => "json",
            EslEventType::ESLEVENTTYPEXML => "xml",
        };
        self.send_recv(format!("event {} {}", event_type, channel_event))
            .await
    }

    async fn send_event(event: Event) {
        todo!()
    }

    pub async fn auth(&mut self) -> Result<Event, EslError> {
        let cmd;
        if self.user.is_empty() {
            cmd = format!("auth {}\n\n", self.password);
        } else {
            cmd = format!("auth {} {}\n\n", self.user, self.password);
        }
        self.send_recv(cmd).await
    }

    //pub async fn subscribe_event(&self) {
    //   tokio::spawn(async move {
    //       self;
    //   });
    //self.transport_rx.next().await
    //}

    pub async fn disconnect(&mut self) -> Result<(), EslError> {
        self.send_recv("exit".to_string()).await?;
        Ok(())
    }
}
