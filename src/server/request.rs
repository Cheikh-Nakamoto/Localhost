use crate::{ get_boundary, get_content_length, remove_prefix, remove_suffix, Config };
use chrono::Utc;
use mio::net::TcpStream;
use mio::Poll;
use regex::Regex;
use urlencoding::decode;
use std::io::{ Error, ErrorKind };
use std::{ collections::HashMap, io::Read };

// -------------------------------------------------------------------------------------
// REQUEST
// -------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Request {
    pub id_session: String,
    pub content_type: String,
    pub content_length: Option<usize>,
    pub location: String,
    pub host: String,
    pub port: u16,
    pub method: String,
    pub head: String,
    pub body: String,
    pub body_byte: Vec<u8>,
    pub filename: String,
    pub length: usize,
    pub reference: String,
    pub boundary: Option<String>,
    pub complete: bool,
    pub headers: HashMap<String, String>,
    pub timestamp: i64,
}

impl Request {
    pub fn new(
        id_session: String,
        content_type: String,
        location: String,
        host: String,
        port: u16,
        method: String,
        head: String,
        body: String,
        body_byte: Vec<u8>,
        filename: String,
        length: usize,
        reference: String
    ) -> Self {
        Self {
            id_session,
            content_type,
            content_length: None,
            location,
            host,
            port,
            method,
            head,
            body,
            body_byte,
            filename,
            length,
            reference,
            boundary: None,
            complete: false,
            headers: HashMap::new(),
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn default() -> Self {
        Request::new(
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            0,
            String::new(),
            String::new(),
            String::new(),
            vec![],
            String::new(),
            0,
            String::new()
        )
    }

    pub fn stream_to_str(stream: &mut TcpStream) -> Result<(String, Vec<u8>), Error> {
        let mut buffer = [0; 8192]; // Buffer de 8 Ko
        let mut request_str = String::new();
        let mut buff_complete = vec![];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    return Err(
                        Error::new(ErrorKind::ConnectionReset, "Connexion fermer par le paire")
                    );
                }
                Ok(n) => {
                    let buff = String::from_utf8_lossy(&buffer[..n]);
                    request_str.push_str(&buff);
                    buff_complete.extend_from_slice(&buffer[..n]);
                }

                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Wait for more data
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                    // Wait for more data
                    continue;
                }
                Err(_) => {
                    // Handle read error
                    return Err(
                        Error::new(ErrorKind::ConnectionReset, "Connexion fermer par le paire")
                    );
                }
            }
        }
        Ok((request_str, buff_complete))
    }

    pub fn read_request(
        stream: &mut TcpStream,
        poll: &mut Poll,
        config: &Config
    ) -> Result<Self, String> {
        let new_line_pattern = "\r\n\r\n";
        let mut request = Request::default();
        let mut is_post = false;

        let mut request_str= String::new();
        let body_byte;
        match Self::stream_to_str(stream) {
            Ok((req, req_byte)) => {
                request_str = req;
                body_byte = req_byte;
            }
            Err(e) => {
                println!("error reading request: {:?}", request_str);
                poll
                    .registry()
                    .deregister(stream)
                    .map_err(|e| e.to_string())?;
                return Err(e.to_string());
            }
        }

        let [is_get, is_delete] = [
            request_str.starts_with("GET"),
            request_str.starts_with("DELETE"),
        ];
        if is_get || is_delete {
            request.complete = true;
            request.method = match is_get {
                true => String::from("GET"),
                false => String::from("DELETE"),
            };
            request.head = request_str.clone();
        } else if request_str.starts_with("POST") {
            is_post = true;
            request.method = String::from("POST");
        } else {
            request.body = request_str.clone();
            request.body_byte = body_byte.clone();
            return Ok(request);
        }

        // Vérification de la présence des 2 parties de la requête
        match request_str.find(new_line_pattern) {
            None => Ok(request),
            Some(header_limit) => {
                let headers = &request_str[..header_limit];

                Request::parse_http_request(headers, &mut request);

                let mut form_data: Vec<HashMap<&str, Option<String>>> = vec![]; // Chaque HashMap représente un champ du formulaire.

                if is_post || is_delete {
                    let mut head = request_str.clone();
                    let mut body = head.split_off(header_limit);
                    body = body.strip_prefix(new_line_pattern).unwrap().to_string();

                    request.head = head.clone();

                    // Size limit
                    let re = Regex::new(r"Content-Length:\s+(?<length>\d+)").unwrap();
                    if let Some(caps) = re.captures(&head) {
                        let content_length = caps["length"].to_string().parse::<usize>().unwrap_or_default();
                        if content_length > config.http.size_limit * 1024 {
                            request.complete = true;
                            request.length = content_length;

                            return Ok(request);
                        }
                    }

                    request.body = body.clone();
                    request.body_byte = body_byte.clone();

                    if let Some(content_length_str) = get_content_length(&head) {
                        match content_length_str.parse::<usize>() {
                            Ok(val) => {
                                request.content_length = Some(val);
                                if body.len() >= val {
                                    request.complete = true;
                                }
                            }
                            Err(_) => (),
                        }
                    }

                    request.boundary = get_boundary(&request_str);
                    let boundary = request.boundary.clone().unwrap_or_default();

                    Self::extract_form_data(&body, boundary, &mut form_data);

                    if let Some(hashmap) = form_data.get(0) {
                        if let Some(Some(file)) = hashmap.get("filename") {
                            request.filename = file.to_string();
                        }
                        if let Some(Some(file)) = hashmap.get("content_type") {
                            request.content_type = file.to_string();
                        }
                    }
                }
                Ok(request)
            }
        }
    }

    pub fn extract_form_data(
        body: &String,
        boundary: String,
        form_data: &mut Vec<HashMap<&str, Option<String>>>
    ) {
        let new_line_pattern = "\r\n\r\n";
        let body_parts = body
            .split(boundary.as_str())
            .map(|s| {
                remove_suffix(remove_prefix(s.to_string(), "\r\n"), "\r\n--").replace(
                    new_line_pattern,
                    "; value="
                )
            })
            .collect::<Vec<String>>();

        // Tu peux jeter un coup d'œil sur la docu pour comprendre la syntaxe
        // https://docs.rs/regex/latest/regex/index.html
        let re = Regex::new(
            r#"(?xs)
                    (?:Content-Disposition:\s*
                    (?<content_disposition>[^;]+);\s*)?
                    (?:name="(?<name>[^"]+)"\s*)?
                    (?:\s*;\s*
                        (?:filename="(?<filename>[^"]+)"\s*)?
                        (?:file_to_delete="(?<file_to_delete>[^"]+)"\s*)?
                        (?:Content-Type:\s*(?<content_type>[^;]+)\s*)?
                    )*
                    ;\s*value=(?<value>.*)?
                    "#
        ).unwrap();

        // Ici on parcourt les différentes parties du body pour voir si les champs recherchés sont là
        body_parts.iter().for_each(|s| {
            if let Some(caps) = re.captures(&s) {
                let mut values = HashMap::new();
                values.insert("content_disposition", Some(caps["content_disposition"].to_string()));

                values.insert("name", Some(caps["name"].to_string()));
                values.insert(
                    "filename",
                    caps.name("filename").map_or(None, |m| Some(m.as_str().to_string()))
                );
                values.insert(
                    "content_type",
                    caps.name("content_type").map_or(None, |m| Some(m.as_str().to_string()))
                );
                values.insert(
                    "file_to_delete",
                    caps.name("file_to_delete").map_or(None, |m| Some(m.as_str().to_string()))
                );
                values.insert("value", Some(caps["value"].to_string()));
                form_data.push(values);
            }
        });
    }

    pub fn parse_http_request(request_str: &str, request: &mut Request) {
        let mut location = String::new();
        let mut host = String::new();
        let mut port: u16 = 0;
        let mut cookie = String::new();
        let mut headers = HashMap::new();

        let lines: Vec<&str> = request_str.lines().collect();

        // Parser la première ligne (ex: "GET /index.html HTTP/1.1")
        if !lines.is_empty() {
            let parts: Vec<&str> = lines[0].split_whitespace().collect();
            if parts.len() >= 2 {
                location = parts[1].to_string(); // URL (/index.html)
            }
        }

        // Parser les en-têtes
        for line in lines.iter().skip(1) {
            if line.starts_with("Host:") {
                let host_parts: Vec<&str> = line.split(":").collect();
                host = host_parts[1].trim().to_string();
                if host_parts.len() > 2 {
                    port = host_parts[2].parse::<u16>().unwrap_or(80);
                }
            } else if line.contains(":") {
                let mut parts = line.splitn(2, ":");
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    let key = key.trim().trim_matches('"').to_string(); // Supprimer les espaces et les guillemets
                    if key == "Cookie" {
                        cookie = value.to_owned();
                    }
                    let value = value.trim().to_string(); // Supprimer les espaces
                    if !key.is_empty() && !value.is_empty() {
                        headers.insert(key, value);
                    }
                }
            }
        }

        let binding = Self::extract_header_value(&lines, "Referer:");
        let referer = binding.split(":").nth(1).unwrap_or_default();

        request.location = location;
        request.id_session = cookie
            .trim()
            .strip_prefix("cookie_01=")
            .unwrap_or_default()
            .to_owned();
        request.host = host;
        request.port = port;
        request.length = request.body.len();
        request.reference = referer.to_string();
    }

    pub fn extract_header_value(headers: &[&str], pattern: &str) -> String {
        let mut header_value = String::new();

        for line in headers {
            if line.starts_with(pattern) {
                let cookie_str = line.trim_start_matches(pattern).trim();
                for cookie in cookie_str.split(';') {
                    let mut parts = cookie.trim().splitn(2, '=');
                    if let (Some(_), Some(value)) = (parts.next(), parts.next()) {
                        header_value = value.to_string();
                    }
                }
            }
        }
        header_value
    }
    pub fn extract_field(request: &Request, fieldname: &str) -> String {
        let mut filename = String::new();
        let mut form_data = vec![];
        if let Some(boundary) = &request.boundary {
            Request::extract_form_data(&request.body, boundary.to_string(), &mut form_data);
        }
        for field in form_data {
            if let Some(Some(name)) = field.get(fieldname) {
                filename = name.to_owned();
            }
        }
        filename
    }
    /*
       La fonction cherche sucessivement le paterne \r\n\r\n puis le bopundary
       et encore \r\n\r\n et separe a chaque fois !
    */
    pub fn extract_values(body: &[u8], boundary: String) -> Vec<u8> {
        let new_line_pattern = b"\r\n\r\n";
        let start_boundary_pattern = format!("\r\n--{}", boundary).into_bytes();
        let start_pos = body
            .windows(new_line_pattern.len())
            .position(|window| window == new_line_pattern)
            .unwrap_or_default();

        let headers_end = start_pos + new_line_pattern.len();
        let start_pos_body = body[headers_end..]
            .windows(start_boundary_pattern.len())
            .position(|window| window == start_boundary_pattern)
            .unwrap_or_default();

        let file_end = headers_end + start_pos_body;
        let tmp = body[headers_end..file_end].to_vec();
        let fist = tmp
            .windows(new_line_pattern.len())
            .position(|window| window == new_line_pattern)
            .unwrap_or_default();
        tmp[fist + 4..].to_vec()
    }

    pub fn uri_decode(&mut self) {
        self.location = match decode(&self.location) {
            Ok(loc) => loc.to_string(),
            Err(_) => self.location.clone(),
        };

        let re = Regex::new(r"^(?<method>[A-Z]+) /(?<location>\S+)").unwrap();
        self.head = re.replace_all(&self.head, format!("$method {}", self.location)).to_string();
    }
}
