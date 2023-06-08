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
/// #plain event example
/// ```
/// Content-Length: 558
/// Content-Type: text/event-plain
/// 
/// Event-Name: BACKGROUND_JOB
/// Core-UUID: b505a8f7-4a5d-4713-8302-1ae56f727110
/// FreeSWITCH-Hostname: Tree
/// FreeSWITCH-Switchname: Tree
/// FreeSWITCH-IPv4: 132.122.237.194
/// FreeSWITCH-IPv6: %3A%3A1
/// Event-Date-Local: 2023-06-08%2009%3A55%3A24
/// Event-Date-GMT: Thu,%2008%20Jun%202023%2001%3A55%3A24%20GMT
/// Event-Date-Timestamp: 1686189324905215
/// Event-Calling-File: mod_event_socket.c
/// Event-Calling-Function: api_exec
/// Event-Calling-Line-Number: 1572
/// Event-Sequence: 817
/// Job-UUID: 78eca064-62f9-49ed-8bb7-e409fd957fab
/// Job-Command: reloadxml
/// Content-Length: 14
/// 
/// OK [Success]
/// ```

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
