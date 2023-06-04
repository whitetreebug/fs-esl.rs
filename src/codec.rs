use std::collections::HashMap;

use bytes::{Buf, BytesMut};
use tokio_util::codec;

use crate::error::EslError;
use crate::event::Event;
use log::{debug, trace};

#[derive(Debug, Clone)]
pub struct EslCodec {}

impl EslCodec {
    pub fn new() -> EslCodec {
        EslCodec {}
    }
}

fn parse_header(src: &bytes::BytesMut) -> Option<usize> {
    for (index, c) in src[..].iter().enumerate() {
        if c == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index);
        }
    }
    None
}

fn conver2map(src: &[u8]) -> HashMap<String, String> {
    let data = String::from_utf8_lossy(&src);
    data.split('\n')
        .map(|line| line.split(':'))
        .map(|mut i| {
            (
                i.next().unwrap().trim().to_string(),
                i.next().unwrap().trim().to_string(),
            )
        })
        .collect()
}

impl codec::Encoder<String> for EslCodec {
    type Error = EslError;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item.as_bytes());
        dst.extend_from_slice(b"\n\n");
        trace!("send: {:?}", dst);
        Ok(())
    }
}

impl codec::Decoder for EslCodec {
    type Item = Event;
    type Error = EslError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        //trace!("recv {:?}", src);
        let mut event: Event = Event::new();
        let header_end_index = parse_header(src);
        if let Some(header_end_index) = header_end_index {
            event.headers = conver2map(&src[..header_end_index]);

            if let Some(content_len) = event.headers.get("Content-Length") {
                let body_len = content_len.parse::<usize>().unwrap();
                if body_len <= src.len() {
                    let body_end_index = header_end_index + 2 + body_len - 2; //remove \n\n
                    event.body = conver2map(&src[header_end_index + 2..body_end_index]);
                    src.advance(body_end_index + 2);
                } else {
                    return Ok(None);
                }
            } else {
                src.advance(header_end_index + 2);
            }
            debug!("recv event: {:?}", event);
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}
