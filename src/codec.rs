use std::collections::HashMap;

use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::error::EslError;
use crate::event::Event;
use log::trace;

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
            return Some(index + 1);
        }
    }
    None
}

impl Encoder<&[u8]> for EslCodec {
    type Error = EslError;

    fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> Result<(), Self::Error> {
        //"Content-Type: auth/request\n\n"
        dst.extend_from_slice(item);
        dst.extend_from_slice(b"\n\n");
        Ok(())
    }
}

impl Decoder for EslCodec {
    type Item = Event;
    type Error = EslError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut event: Event = Event::new();
        let header_end_index = parse_header(src);
        if header_end_index.is_none() {
            return Ok(None);
        }

        if let Some(header_end_index) = header_end_index {
            let data = String::from_utf8_lossy(&src[..header_end_index - 1]);
            event.headers = data
                .split('\n')
                .map(|line| line.split(':'))
                .map(|mut i| {
                    (
                        i.next().unwrap().trim().to_string(),
                        i.next().unwrap().trim().to_string(),
                    )
                })
                .collect();

            let body_start_index = header_end_index + 1;

            if let Some(length) = event.headers.get("Content-Length") {
                let content_length = length.parse::<usize>()?;
                if src.len() < (header_end_index + content_length + 1) {}
                event.body = Some(String::from_utf8_lossy(&src[body_start_index..]).to_string());
                src.advance(body_start_index + content_length);
            } else {
                src.advance(body_start_index); //修改内部指针位置
            };
        }
        trace!("recv: {:?}", event);
        Ok(Some(event))
    }
}
