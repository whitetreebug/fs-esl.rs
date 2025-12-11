use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Event {
    pub header: HashMap<String, String>,
    pub body: String,
}

impl Event {
    pub fn header(&self) -> &HashMap<String, String> {
        &self.header
    }

    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        match self.header.get(key) {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }

    pub fn get_type(&self) -> Option<&str> {
        match self.header.get("Event-Name") {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }
    pub fn add_header(&mut self, key: &str, val: &str) {
        self.header.insert(key.to_string(), val.to_string());
    }

    pub fn add_body(&mut self, body: &str) {
        self.body = body.to_string()
    }

    pub fn parse_body(body: &str) -> Option<String> {
        let space_index = body
            .find(char::is_whitespace)
            .expect("can not find _body or parse _body error");
        let text_start = space_index + 1;
        let body_length = body.len();
        if text_start < body_length {
            Some(body[text_start..(body_length - 1)].to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct EslMsg {
    pub header: HashMap<String, String>,
    pub event: Event,
}

impl EslMsg {
    pub fn ref_event(&self) -> &Event {
        &self.event
    }

    pub fn event(&self) -> Event {
        self.event.clone()
    }
}

/*
    "CUSTOM",
    "CLONE",
    "CHANNEL_CREATE",
    "CHANNEL_DESTROY",
    "CHANNEL_STATE",
    "CHANNEL_CALLSTATE",
    "CHANNEL_ANSWER",
    "CHANNEL_HANGUP",
    "CHANNEL_HANGUP_COMPLETE",
    "CHANNEL_EXECUTE",
    "CHANNEL_EXECUTE_COMPLETE",
    "CHANNEL_HOLD",
    "CHANNEL_UNHOLD",
    "CHANNEL_BRIDGE",
    "CHANNEL_UNBRIDGE",
    "CHANNEL_PROGRESS",
    "CHANNEL_PROGRESS_MEDIA",
    "CHANNEL_OUTGOING",
    "CHANNEL_PARK",
    "CHANNEL_UNPARK",
    "CHANNEL_APPLICATION",
    "CHANNEL_ORIGINATE",
    "CHANNEL_UUID",
    "API",
    "LOG",
    "INBOUND_CHAN",
    "OUTBOUND_CHAN",
    "STARTUP",
    "SHUTDOWN",
    "PUBLISH",
    "UNPUBLISH",
    "TALK",
    "NOTALK",
    "SESSION_CRASH",
    "MODULE_LOAD",
    "MODULE_UNLOAD",
    "DTMF",
    "MESSAGE",
    "PRESENCE_IN",
    "NOTIFY_IN",
    "PRESENCE_OUT",
    "PRESENCE_PROBE",
    "MESSAGE_WAITING",
    "MESSAGE_QUERY",
    "ROSTER",
    "CODEC",
    "BACKGROUND_JOB",
    "DETECTED_SPEECH",
    "DETECTED_TONE",
    "PRIVATE_COMMAND",
    "HEARTBEAT",
    "TRAP",
    "ADD_SCHEDULE",
    "DEL_SCHEDULE",
    "EXE_SCHEDULE",
    "RE_SCHEDULE",
    "RELOADXML",
    "NOTIFY",
    "PHONE_FEATURE",
    "PHONE_FEATURE_SUBSCRIBE",
    "SEND_MESSAGE",
    "RECV_MESSAGE",
    "REQUEST_PARAMS",
    "CHANNEL_DATA",
    "GENERAL",
    "COMMAND",
    "SESSION_HEARTBEAT",
    "CLIENT_DISCONNECTED",
    "SERVER_DISCONNECTED",
    "SEND_INFO",
    "RECV_INFO",
    "RECV_RTCP_MESSAGE",
    "CALL_SECURE",
    "NAT",
    "RECORD_START",
    "RECORD_STOP",
    "PLAYBACK_START",
    "PLAYBACK_STOP",
    "CALL_UPDATE",
    "FAILURE",
    "SOCKET_DATA",
    "MEDIA_BUG_START",
    "MEDIA_BUG_STOP",
    "CONFERENCE_DATA_QUERY",
    "CONFERENCE_DATA",
    "CALL_SETUP_REQ",
    "CALL_SETUP_RESULT",
    "CALL_DETAIL",
    "DEVICE_STATE",
    "TEXT",
    "ALL"
*/
