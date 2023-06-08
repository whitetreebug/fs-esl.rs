use std::collections::HashMap;

use bytes::{Buf, BytesMut};
use tokio_util::codec;

use crate::event::Event;
use crate::{error::EslError, event::EslMsg};
use log::{debug, trace};

#[derive(Debug, Clone)]
pub struct EslCodec {}

impl EslCodec {
    pub fn new() -> EslCodec {
        EslCodec {}
    }
}

pub fn find_crlfcrlf(src: &[u8]) -> Option<usize> {
    for (index, c) in src[..].iter().enumerate() {
        if c == &b'\n' && src.get(index + 1) == Some(&b'\n') {
            return Some(index);
        }
    }
    None
}

pub fn conver2map(src: &[u8]) -> HashMap<String, String> {
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
        trace!("recv {:?}", src);
        let mut esl_msg = EslMsg::new();
        let msg_header_end_index = find_crlfcrlf(src);
        if let Some(msg_header_end_index) = msg_header_end_index {
            esl_msg.header = conver2map(&src[..msg_header_end_index]);
            trace!("esl_msg.header: {:?}", esl_msg.header);
            let msg_header_end_index = msg_header_end_index + 2; //\n\n
            if let Some(raw_event_len) = esl_msg.header.get("Content-Length") {
                let raw_event_len = raw_event_len.parse::<usize>().unwrap();
                if raw_event_len <= src.len() {
                    let raw_event_src =
                        &src[msg_header_end_index..msg_header_end_index + raw_event_len];
                    if let Some(raw_event_header_end_index) = find_crlfcrlf(raw_event_src) {
                        esl_msg.event.header =
                            conver2map(&raw_event_src[..raw_event_header_end_index]);
                        if let Some(raw_event_body_len) = esl_msg.event.header.get("Content-Length")
                        {
                            let raw_event_body_len = raw_event_body_len.parse::<usize>().unwrap();
                            let raw_event_header_end_index = raw_event_header_end_index + 2; //\n\n
                            esl_msg.event.body = String::from_utf8_lossy(
                                &raw_event_src[raw_event_header_end_index
                                    ..(raw_event_header_end_index + raw_event_body_len)],
                            )
                            .to_string();
                        }
                    } else {
                        esl_msg.event.header = conver2map(&raw_event_src);
                    }
                    src.advance(msg_header_end_index + raw_event_src.len());
                } else {
                    return Ok(None);
                }
            } else {
                src.advance(msg_header_end_index);
            }
            debug!("recv event: {:?}", esl_msg);
            Ok(Some(esl_msg.event))
        } else {
            Ok(None)
        }
    }
}
