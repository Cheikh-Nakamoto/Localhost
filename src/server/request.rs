use std::io::Read;

use mio::{net::TcpStream, Token};

// -------------------------------------------------------------------------------------
// REQUEST
// -------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Request {
    pub id_session: String,
    pub location: String,
    pub host: String,
    pub port: u16,
    pub method: String,
    pub bytes: usize,
    pub body: String,
    pub filename: String,
    pub lenght: usize,
}

impl Request {
    pub fn new(
        id_session: String,
        location: String,
        host: String,
        port: u16,
        method: String,
        bytes: usize,
        body: String,
        filename: String,
        lenght: usize,
    ) -> Self {
        Self {
            id_session,
            location,
            host,
            port,
            method,
            bytes,
            body,
            filename,
            lenght,
        }
    }
    /// Lit une requête HTTP à partir d'un TcpStream.
    pub fn read_request(stream: &mut TcpStream, token: Token) -> Self {
        let mut buffer = [0; 8192]; // Buffer de 8 Ko
        let mut request_str = String::new();
        let mut headers_end = None;

        // Lire les données du client jusqu'à la fin des en-têtes
        loop {
            let n = stream.read(&mut buffer).unwrap_or_default();
            if n == 0 {
                // Connexion fermée par le client
                eprintln!("Client({}) déconnecté", token.0);
                break;
            }

            // Convertir les données reçues en une chaîne de caractères
            request_str.push_str(&String::from_utf8_lossy(&buffer[..n]));

            // Vérifier si la fin des en-têtes a été atteinte
            if let Some(pos) = request_str.find("\r\n\r\n") {
                headers_end = Some(pos);
                break;
            }
        }

        // Extraire les en-têtes
        let headers_end = headers_end.unwrap_or_default();
        let headers = &request_str[..headers_end];

        // Parser les en-têtes pour créer une instance de `Request`
        let request = Request::parse_http_request(headers);

        request
    }
    /// Parse une requête HTTP et crée une instance de `Request`.
    pub fn parse_http_request(request_str: &str) -> Self {
        let mut location = String::new();
        let mut host = String::new();
        let mut port: u16 = 0;
        let mut method = String::new();
        let mut filename = String::new();
        let mut body = String::new();
        let mut length = 0;

        // Diviser la requête en lignes
        let lines: Vec<&str> = request_str.lines().collect();

        // Parser la première ligne (ex: "GET /index.html HTTP/1.1")
        if !lines.is_empty() {
            let parts: Vec<&str> = lines[0].split_whitespace().collect();
            if parts.len() >= 2 {
                method = parts[0].to_string(); // Méthode (GET, POST, etc.)
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
            } else if line.starts_with("Content-Disposition:") {
                // Extraire le nom du fichier
                filename = Self::extract_filename(line);
            } else if line.starts_with("Content-Length:") {
                // Extraire la taille du fichier
                length = line
                    .split(":")
                    .nth(1)
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0);
            }
        }

        // Parser le corps de la requête (s'il existe)
        let mut is_body = false;
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                // Une ligne vide sépare les en-têtes du corps
                is_body = true;
                continue;
            }
            if is_body {
                body.push_str(line);
                body.push('\n');
            }
        }

        // Créer une instance de `Request`
        Request::new(
            String::new(), // id_session (à remplir plus tard)
            location,
            host,
            port,
            method,
            0,
            body,
            filename,
            length,
        )
    }

    fn extract_filename(header: &str) -> String {
        let parts: Vec<&str> = header.split("filename=").collect();
        if parts.len() > 1 {
            parts[1]
                .trim_matches('"')
                .trim_matches(';')
                .trim()
                .to_string()
        } else {
            String::new()
        }
    }
    pub fn extract_header_value(headers: &[&str], paterne: &str) -> String {
        let mut header_value = String::new();

        for line in headers {
            if line.starts_with(paterne) {
                let cookie_str = line.trim_start_matches(paterne).trim();
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
}
// -------------------------------------------------------------------------------------
