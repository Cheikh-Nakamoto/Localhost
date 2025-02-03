extern crate core;

pub mod server;
use std::{ collections::HashMap, fs, io::Error };

use regex::Regex;
pub use server::*;

use colored::Colorize;

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

pub fn verify_config(config: &mut Config) -> std::io::Result<()> {
    if !config.http.servers.is_empty() {
        for (name, server) in config.http.servers.clone().iter() {
            if !is_valid_ip(&server.ip_addr) {
                println!(
                    "{}",
                    format!("L'adresse IP {} est invalide ou indisponible", server.ip_addr)
                        .red()
                        .bold()
                );
                config.http.servers.remove(name);
            }
        }
    }

    match config.http.servers.is_empty() {
        true => Err(Error::new(std::io::ErrorKind::InvalidData, "Aucune adresse IP valide.")),
        false => Ok(())
    }
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

// pub fn is_available_ip(ip: &str) -> bool {
//     // Validation de base avec regex
//     let ipv4_re = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])$").unwrap();

//     if !ipv4_re.is_match(ip) {
//         return false;
//     }

//     // Conversion en Ipv4Addr pour les vérifications
//     let octets: Vec<u8> = ip.split('.')
//         .map(|s| s.parse().unwrap())
//         .collect();

//     // Vérification des plages réservées
//     let is_private = octets[0] == 10 ||                               // 10.0.0.0/8
//                     (octets[0] == 172 && (16..=31).contains(&octets[1])) || // 172.16.0.0/12
//                     (octets[0] == 192 && octets[1] == 168);           // 192.168.0.0/16

//     let is_loopback = octets[0] == 127;                              // 127.0.0.0/8
//     let is_link_local = octets[0] == 169 && octets[1] == 254;       // 169.254.0.0/16
//     let is_multicast = octets[0] >= 224 && octets[0] <= 239;        // 224.0.0.0/4
//     let is_broadcast = ip == "255.255.255.255";                     // Broadcast global
//     let is_zeroconf = ip == "0.0.0.0";                              // Adresse invalide

//     // Retourne false si aucune de ces conditions n'est remplie
//     !(is_private || is_loopback || is_link_local || is_multicast || is_broadcast || is_zeroconf)
// }

pub fn is_valid_ip(ip: &str) -> bool {
    // Pattern détaillé pour IPv4
    let ipv4_pattern =
        r#"^((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.){3}(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)$"#;

    Regex::new(ipv4_pattern).unwrap().is_match(ip)
}
