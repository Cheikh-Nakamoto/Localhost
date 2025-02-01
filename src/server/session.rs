use chrono::{DateTime, Duration, Utc};
use mio::net::TcpStream;
use std::collections::HashMap;
use std::io::{BufReader, Read};
use uuid::Uuid;

// -------------------------------------------------------------------------------------
// SESSION
// -------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub validity_time: DateTime<Utc>,
    pub expiration_time: i64,
}

impl Session {
    pub(crate) const SESSION_LIFETIME: i64 = 60 * 60 * 1000;

    pub fn new() -> Self {
        let expires_duration = Duration::milliseconds(Self::SESSION_LIFETIME);

        Self {
            id: Uuid::new_v4().into(),
            expiration_time: Self::SESSION_LIFETIME,
            validity_time: Utc::now() + expires_duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.validity_time
    }

    /// Récupère la valeur d'un cookie spécifique à partir d'un TcpStream.
    pub fn get_cookie_from_stream(stream: &mut TcpStream, cookie_name: &str) -> Option<String> {
        // Crée un BufReader pour lire les données du flux
        let mut reader = BufReader::new(stream);

        // Lit les en-têtes HTTP
        let mut headers = String::new();
        let mut buffer = [0; 1024];

        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break; // Fin du flux
            }

            // Ajoute les données lues à la chaîne d'en-têtes
            headers.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

            // Vérifie si la fin des en-têtes a été atteinte (double saut de ligne)
            if headers.contains("\r\n\r\n") {
                break;
            }
        }

        // Analyse les en-têtes pour extraire le cookie
        Self::extract_cookie_from_headers(&headers, cookie_name)
    }

    /// Extrait la valeur d'un cookie spécifique à partir des en-têtes HTTP.
    fn extract_cookie_from_headers(headers: &str, cookie_name: &str) -> Option<String> {
        // Parcourt chaque ligne des en-têtes
        for line in headers.lines() {
            // Vérifie si la ligne contient l'en-tête "Cookie"
            if line.starts_with("Cookie:") {
                // Extrait la partie après "Cookie:"
                let cookies_str = line.trim_start_matches("Cookie:").trim();

                // Divise les cookies en paires clé-valeur
                let cookies: HashMap<String, String> = cookies_str
                    .split(';')
                    .filter_map(|cookie| {
                        let mut parts = cookie.trim().split('=');
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            Some((key.to_string(), value.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                // Retourne la valeur du cookie demandé
                return cookies.get(cookie_name).cloned();
            }
        }

        None // Cookie non trouvé
    }

    pub fn make_cookie(cookie_name: &str, cookie_value: &str, expires_in: i64) -> String {
        // Convertir expires_in (millisecondes) en Duration
        let expires_duration = Duration::milliseconds(expires_in);

        // Calculer la date d'expiration
        let expires = (Utc::now() + expires_duration)
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();

        // Construire l'en-tête Set-Cookie
        let cookie_header = format!(
            "Set-Cookie: {}={}; Path=/; HttpOnly; Expires={}\r\n",
            cookie_name, cookie_value, expires
        );

        cookie_header
    }
}
// -------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_session_not_expired() {
        let session = Session::new();
        assert!(
            !session.is_expired(),
            "La session ne devrait pas être expirée."
        );
    }

    #[test]
    fn test_session_expired() {
        let mut session = Session::new();
        session.expiration_time = Utc::now().timestamp_millis() - 1000; // Expiration dans le passé
        assert!(session.is_expired(), "La session devrait être expirée.");
    }

    #[test]
    fn test_session_expired_after_delay() {
        let session = Session::new();
        thread::sleep(Duration::from_secs(2)); // Attendre 2 secondes
        assert!(
            session.is_expired(),
            "La session devrait être expirée après le délai."
        );
    }
}
