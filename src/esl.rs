use crate::{codec::EslCodec, error::EslError, event::Event};
use futures::SinkExt;
use log::{error, trace};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{self, AsyncWriteExt, Interest, ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc,
    time,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, FramedWrite};

pub struct EslHandle {
    user: String,
    password: String,
    transport_rx: Arc<Mutex<FramedRead<ReadHalf<TcpStream>, EslCodec>>>,
    transport_tx: Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, EslCodec>>>,
   // channel_rx: Receiver<String>,
}

impl EslHandle {
    pub async fn connect(
        user: &str,
        password: &str,
        addr: impl ToSocketAddrs,
    ) -> Result<EslHandle, EslError> {
        let stream: TcpStream = TcpStream::connect(addr).await?;
       // let (tx, mut rx) = mpsc::channel(1000);
        let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await.unwrap();
        let (orh, owh) = stream.into_split();
        /* 
        stream.into_split()
        let (rh, wh) = io::split(stream);
        let mut transport_rx = Arc::new(Mutex::new(FramedRead::new(rh, EslCodec::new())));
        let mut transport_tx = Arc::new(Mutex::new(FramedWrite::new(wh, EslCodec::new())));
        let handle = EslHandle {
            user: user.to_string(),
            password: password.to_string(),
            transport_rx: transport_rx,
            transport_tx: transport_tx,
         //   channel_rx: rx,
        };
        */
        handle.auth().await;
        loop {
        }
        // tokio::spawn(async move {
        //    loop {
        /*
            match self.transport_rx.next().await {
                None => {
                    //   trace!("recv message unkonw error"),
                }
                Some(Err(e)) => {
                    //   trace!("recv message error: {:?}", e.to_string()),
                }
                Some(Ok(event)) => {
                    //println!("cccccccccccc");
                    //if tx.send(event).await.is_err() {
                    //   error!("send event error");
                }
            }
        }
        */
        //}).await;

        /*
        let (tx, mut rx) = mpsc::channel::<Event>(100);


                tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                trace!("{:?}", event);
                match event.get_val("Content-Type").unwrap_or_default() {
                    "auth/request" => {
                        println!("cccccccccccc");
                        trace!("{:?}", event);
                        if transport_tx.send(b"auth ClueCon").await.is_err() {
                            error!("send to server error");
                        }
                    }
                    _ => trace!("other event: {:?}", event),
                }
            }
        });
        */
        Ok(handle)
    }

    /*pub async fn recv(mut rx: FramedRead<ReadHalf<TcpStream>, EslCodec>)
    {
    }

    pub async fn send()
    {

    }
    */
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

    pub async fn auth(&self) {
        let cmd;
        if self.user.is_empty() {
            cmd = format!("auth {}\n\n", self.password);
        } else {
            cmd = format!("auth {} {}\n\n", self.user, self.password);
        }

        trace!("{:?}", cmd);
        self.send(cmd.as_bytes()).await;
    }

    pub async fn send(&self, item: &[u8]) {
        let mut transport_tx = self.transport_tx.lock().unwrap();
        transport_tx.send(item).await;
    }

    //pub async fn subscribe_event(&self) {
    //   tokio::spawn(async move {
    //       self;
    //   });
    //self.transport_rx.next().await
    //}
}
