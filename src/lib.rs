pub mod server;
use std::{ collections::HashMap, fs };

use regex::Regex;
pub use server::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub log_files: LogFilesConfig,
    pub http: HttpConfig,
}

impl Config {
    pub fn new() -> Self {
        Self {
            log_files: LogFilesConfig {
                error_log: String::new(),
                access_log: String::new(),
                events_limit: 0,
            },
            http: HttpConfig {
                access_log_format: String::new(),
                timeout: 0,
                size_limit: 0,
                servers: HashMap::new(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogFilesConfig {
    pub error_log: String,
    pub access_log: String,
    pub events_limit: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub access_log_format: String,
    pub timeout: u64,
    pub size_limit: usize,
    pub servers: HashMap<String, Server>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Redirection {
    pub source: String,
    pub target: String,
}

pub fn load_config() -> Config {
    let content = fs::read_to_string("src/config.toml").unwrap_or(String::new());
    toml::from_str(&content).unwrap()
}

pub fn remove_suffix(str: String, suffix: &str) -> String {
    match str.strip_suffix(suffix) {
        Some(txt) => txt.to_string(),
        None => str,
    }
}

pub fn remove_prefix(str: String, prefix: &str) -> String {
    match str.strip_prefix(prefix) {
        Some(txt) => txt.to_string(),
        None => str,
    }
}

pub fn get_boundary(req: &String) -> Option<String> {
    let re = Regex::new(r"boundary=(?<var_limit>[-_a-zA-Z0-9]+)\r").unwrap();
    if let Some(caps) = re.captures(&req) {
        Some(caps["var_limit"].to_string())
    } else {
        return None;
    }
}

pub fn get_content_length(req: &String) -> Option<String> {
    let re = Regex::new(r"Content-Length:\s*(?<content_type>\d+)").unwrap();
    if let Some(caps) = re.captures(&req) {
        Some(caps["content_type"].to_string())
    } else {
        return None;
    }
}