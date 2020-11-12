use serde_json::Value;
use std::collections::HashMap;

pub struct Profile {
    pub input: Protocol
}

pub struct Protocol {
    /// protocal name
    pub name: String,
    /// Config
    pub config: Value,
}