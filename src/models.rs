use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub url: String,
    pub port: u16,
    pub query_params: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub payload: Option<String>,
    pub method: String,
}

impl Connection {
    pub fn new(name: String, url: String, port: u16) -> Self {
        Self {
            name,
            url,
            port,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            payload: None,
            method: "GET".to_string(),
        }
    }

    pub fn full_url(&self) -> String {
        // Use HTTPS for port 443, HTTP for others
        let protocol = if self.port == 443 { "https" } else { "http" };
        let mut url = format!("{}://{}:{}", protocol, self.url, self.port);
        if !self.query_params.is_empty() {
            url.push('?');
            let params: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&params.join("&"));
        }
        url
    }
}

#[derive(Debug, Clone)]
pub enum InputMode {
    Normal,
    ConnectionName,
    EditingConnection,
    EditingPayload,
    EditingKeyValue,
    Connecting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Connections,
    Response,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditField {
    Name,
    Url,
    Port,
    Method,
    Headers,
    QueryParams,
    Payload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyValueTarget {
    Headers,
    QueryParams,
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: String,
}
