use serde_json::Value;
use std::collections::HashMap;

pub struct Profile {
    pub input: Protocol
}

pub struct Protocol {
    pub name: String,
    //
    pub config: Value,
}