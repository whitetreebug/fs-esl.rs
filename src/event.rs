use std::collections::HashMap;

#[derive(Debug)]
pub struct Event {
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Event {
    pub fn new() -> Event {
        Event {
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &Option<String> {
        &self.body
    }

    pub fn get_val(&self, key: &str) -> Option<&str> {
        match self.headers.get(key) {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }
}
