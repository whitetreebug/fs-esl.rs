use std::collections::HashMap;

#[derive(Debug)]
pub struct Event {
    pub headers: HashMap<String, String>,
    pub body: HashMap<String,String>,
    //pub events: HashMap<String, String>,
}

impl Event {
    pub fn new() -> Event {
        Event {
            headers: HashMap::new(),
            body: HashMap::new(),
        }
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &HashMap<String, String> {
        &self.body
    }


    pub fn get_val(&self, key: &str) -> Option<&str> {
        match self.headers.get(key) {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }
}
