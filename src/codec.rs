use bytes::{Buf, BytesMut};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use tokio_util::codec;

use crate::esl::EslEventType;
use crate::event::{self, Event};
use crate::{error::EslError, event::EslMsg};
use log::{error, info, trace, warn};

#[derive(Debug, Default, Clone)]
pub struct EslCodec {}

pub fn find_header_end(src: &[u8]) -> Option<(usize, usize)> {
    if let Some(start) = src.windows(4).position(|window| window == b"\r\n\r\n") {
        return Some((start, start + 4));
    }

    if let Some(start) = src.windows(2).position(|window| window == b"\n\n") {
        return Some((start, start + 2));
    }
    None
}

pub fn convert2map(src: &[u8], event_type: EslEventType) -> HashMap<String, String> {
    let mut res = HashMap::new();
    match event_type {
        EslEventType::PLAIN => {
            let data = String::from_utf8_lossy(src);
            res = data
                .split('\n')
                .filter_map(|line| line.split_once(':'))
                .map(|(key, value)| (key.trim(), value.trim()))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                // .map(|(key, value)| (Cow::Borrowed(key), Cow::Borrowed(value)))
                .collect::<HashMap<String, String>>();
        }
        EslEventType::JSON => {
            if let Ok(json) = serde_json::from_slice::<Value>(src) {
                match serde_json::from_value::<HashMap<String, String>>(json) {
                    Ok(tmp) => res = tmp,
                    Err(e) => error!("parse msg event to json error{:?}", e),
                }
            } else {
                error!("parse msg event to json error");
            }
        }
        EslEventType::XML => {}
    }

    res
}

pub fn parse_plain(raw_event_src: &[u8]) -> Event {
    let mut event = Event::default();
    if let Some((index_start, index_end)) = find_header_end(raw_event_src) {
        event.header = convert2map(&raw_event_src[..index_start], EslEventType::PLAIN);
        if let Some(raw_event_body_len) = event.header.get("Content-Length") {
            if let Ok(raw_event_body_len) = raw_event_body_len.parse::<usize>() {
                event.body = String::from_utf8_lossy(
                    &raw_event_src[index_end..(index_end + raw_event_body_len)],
                )
                .to_string();
                event.header.insert("_body".to_string(), event.body.clone());
            } else {
                error!("parse body Content-Length failed {:?}", raw_event_src);
            }
        }
    } else {
        event.header = convert2map(raw_event_src, EslEventType::PLAIN);
    }
    event
}

pub fn parse_xml(_raw_event_src: &[u8]) -> Event {
    warn!("todo!");
    event::Event::default()
}

pub fn parse_json(raw_event_src: &[u8]) -> Event {
    let mut event = Event::default();
    if let Some((index_start, index_end)) = find_header_end(raw_event_src) {
        event.header = convert2map(&raw_event_src[..index_start], EslEventType::JSON);
        if let Some(raw_event_body_len) = event.header.get("Content-Length") {
            if let Ok(raw_event_body_len) = raw_event_body_len.parse::<usize>() {
                event.body = String::from_utf8_lossy(
                    &raw_event_src[index_end..(index_end + raw_event_body_len)],
                )
                .to_string();
            } else {
                error!("parse body Content-Length failed {:?}", raw_event_src);
            }
        }
    } else {
        event.header = convert2map(raw_event_src, EslEventType::JSON);
    }

    event
}

impl codec::Encoder<&[u8]> for EslCodec {
    type Error = EslError;

    fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        dst.extend_from_slice(b"\n\n");
        trace!("send: {:?}", dst);
        Ok(())
    }
}
/// #plain event example
/// ```text
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
        info!("recv\n{:?}", src);
        let res = match find_header_end(src) {
            Some((index_start, index_end)) => {
                let mut esl_msg = EslMsg {
                    header: convert2map(&src[..index_start], EslEventType::PLAIN),
                    event: Event::default(),
                };
                trace!("esl_msg.header: {:?}", esl_msg.header);

                if let Some(raw_event_len) = esl_msg.header.get("Content-Length") {
                    if let Ok(raw_event_len) = raw_event_len.parse::<usize>() {
                        if raw_event_len <= src.len() {
                            let raw_event_src = &src[index_end..index_end + raw_event_len];
                            if let Some(content_type) = esl_msg.header.get("Content-Type") {
                                match content_type.as_str() {
                                    "text/event-plain" => {
                                        esl_msg.event = parse_plain(raw_event_src)
                                    }
                                    "text/event-json" => esl_msg.event = parse_json(raw_event_src),
                                    "text/event-xml" => esl_msg.event = parse_json(raw_event_src),
                                    "text/rude-rejection" | "text/disconnect-notice" => {
                                        return Err(EslError::AccessDeniedError(
                                            "ACL Refuse".to_string(),
                                        ));
                                    }
                                    _ => {
                                        error!("more condition need to deal, {:?}", &esl_msg.header)
                                    }
                                }
                            }

                            src.advance(index_end + raw_event_src.len());
                        } else {
                            return Ok(None);
                        }
                    } else {
                        error!("parse header Content-Length failed {:?}", esl_msg.header);
                    }
                } else {
                    src.advance(index_end);
                }
                trace!("recv event: {:?}", esl_msg);
                Ok(Some(esl_msg.event))
            }
            None => Ok(None),
        };

        res
    }
}
