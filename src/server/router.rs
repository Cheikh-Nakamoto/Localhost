use crate::Config;
use super::Request;
pub use super::{Server, Session};
use hostfile::{get_hostfile_path, parse_hostfile, HostEntry};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Error, ErrorKind, Write};
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

// -------------------------------------------------------------------------------------
// ROUTER
// -------------------------------------------------------------------------------------
const CLIENT_START: Token = Token(1000); // Token de départ pour les clients

#[derive(Debug)]
pub struct Router {
    pub servers: Vec<Server>,
    pub sessions: HashMap<Token, Session>,
    pub listeners: HashMap<Token, TcpListener>, // Associe un token à un TcpListener
    pub clients: HashMap<Token, TcpStream>,     // Associe un token à un TcpStream
    pub next_token: usize,
    pub request_queue: Vec<Request>,
    pub conn_timeout: HashMap<TcpStream, Instant>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            servers: vec![],
            sessions: HashMap::new(),
            listeners: HashMap::new(),
            clients: HashMap::new(),
            next_token: CLIENT_START.0,
            request_queue: vec![],
            conn_timeout: HashMap::new(),
        }
    }

    /// Ajoute un serveur et démarre l'écoute sur ses ports.
    pub fn add_server(&mut self, server: Server) -> io::Result<()> {
        for &port in &server.ports {
            // Utiliser le hostname si l'adresse IP existe déjà, sinon utiliser l'adresse IP
            let ip_adres= server.ip_addr.trim();
            let addr = if self.ip_addr_existe(&server) {
                server.hostname.trim()
            } else {
                ip_adres
            };

            // Résoudre l'adresse
            let socket_addr = format!("{}:{}", addr, port)
                .to_socket_addrs()
                .map_err(|e| {
                    eprintln!(
                        "Erreur de résolution de l'adresse {}:{} : {}",
                        addr, port, e
                    );
                    io::Error::new(io::ErrorKind::Other, "Failed to resolve address")
                })?
                .next()
                .ok_or_else(|| {
                    eprintln!("Aucune adresse trouvée pour {}:{}", addr, port);
                    io::Error::new(io::ErrorKind::Other, "No address found")
                })?;
            // Lier le TcpListener à l'adresse
            if let Ok(listener) = TcpListener::bind(socket_addr) {
                let token = Token(self.next_token - 1000);
                self.next_token += 1;
                self.listeners.insert(token, listener);
            } else {
                eprintln!("socket_adresse: {} , est deja lié", socket_addr);
            }
        }
        self.servers.push(server);
        Ok(())
    }
    fn ip_addr_existe(&self, server: &Server) -> bool {
        self.servers
            .iter()
            .any(|serv| serv.ip_addr == server.ip_addr)
    }

    pub fn remove_server(&mut self, server: Server) -> io::Result<()> {
        // Filtrer les serveurs pour supprimer celui qui correspond
        self.servers
            .retain(|s| s.ip_addr != server.ip_addr && s.hostname != server.hostname);

        // Fermer les listeners associés à ce serveur
        for &port in &server.ports {
            // Trouver le token associé à ce port
            let token = self
                .listeners
                .iter()
                .find(|(_, listener)| {
                    listener.local_addr().ok().map(|addr| addr.port()) == Some(port)
                })
                .map(|(token, _)| *token);
            if let Some(token) = token {
                // Fermer le listener en le retirant de la HashMap
                self.listeners.remove(&token);
                println!("Listener fermé pour le port {}", port);
            }
        }

        Ok(())
    }
    /// Démarre le Router et commence à écouter les événements.
    pub fn run(&mut self, config: &Config) -> io::Result<()> {
        let mut poll = Poll::new()?;
        let mut server_tokens = HashMap::new();

        // Enregistrer chaque listener avec un token unique
        for (token, listener) in &mut self.listeners {
            poll.registry()
                .register(listener, *token, Interest::READABLE | Interest::WRITABLE)?;
            server_tokens.insert(*token, listener.local_addr()?);
        }

        let mut events = Events::with_capacity(config.log_files.events_limit);

        loop {
            poll.poll(&mut events, None)?;

            for event in events.iter() {
                if let Some(_) = server_tokens.get(&event.token()) {
                    // Nouvelle connexion sur un TcpListener
                    self.accept_connection(event.token(), &poll)?;
                    // println!("Nouvelle connexion sur le port {}", addr.port());
                } else if event.is_readable() {
                    // Données reçues sur un TcpStream
                    let stream = self
                        .clients
                        .get_mut(&event.token())
                        .expect("Erreur lors de la recupération du canal tcpstream");
                    let mut req = Request::default();
                    match Request::read_request(stream, &mut poll) {
                        Ok(request) => {
                            req = request;
                        }
                        Err(_) => {
                            // dbg!("suppresion du client dans self.client");
                            // dbg!("Error found", e);
                            // Fermer le stream proprement
                            if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
                                eprintln!("Erreur lors de la fermeture du stream: {}", e);
                            }
                            self.clients.remove(&event.token());
                            continue;
                        }
                    };

                    req.uri_decode();

                    let mut cookie = req.id_session.clone();
                    // println!("cookie extract: {}",cookie);
                    let client_token = Token(self.next_token);
                    self.next_token += 1;
                    // Tentative de récupération du cookie
                    if !cookie.is_empty() {
                        // Recherche d'une session existante avec le même cookie
                        let mut session_found = false;

                        for (old_token, session) in self.sessions.clone().iter() {
                            if session.id.trim() == cookie && !session.is_expired() {
                                let mut new_session = Session::new();
                                new_session.id = session.id.clone();
                                self.sessions.remove(&old_token);
                                self.sessions.insert(client_token.clone(), new_session);
                                session_found = true;
                                break;
                            }
                        }

                        if !session_found {
                            // Si aucune session existante n'est trouvée, créez une nouvelle session
                            let new_session = Session::new();
                            self.sessions
                                .insert(client_token.clone(), new_session.clone());
                        }
                    } else {
                        // Si aucun cookie n'est trouvé, créez une nouvelle session
                        let new_session = Session::new();
                        self.sessions
                            .insert(client_token.clone(), new_session.clone());
                    }

                    if let Some(session) = self.sessions.get_mut(&client_token) {
                        cookie =
                            Session::make_cookie("cookie_01", &session.id, session.expiration_time);
                    }

                    if ["GET", "POST", "DELETE"].contains(&req.method.as_str()) {
                        self.request_queue.push(req.clone());
                    } else {
                        for (i, waiting_req) in self.request_queue.clone().iter().enumerate() {
                            if ["POST", "DELETE"].contains(&waiting_req.method.as_str()) {
                                if let Some(content_length) = waiting_req.content_length {
                                    if content_length > waiting_req.body.len() {
                                        if let Some(boundary) = waiting_req.boundary.clone() {
                                            if req.body.contains(&boundary) {
                                                self.request_queue[i].body.push_str(&req.body);
                                                self.request_queue[i]
                                                    .body_byte
                                                    .extend_from_slice(&req.body_byte);
                                                if let Some(content_length) =
                                                    self.request_queue[i].content_length
                                                {
                                                    if self.request_queue[i].body.len()
                                                        >= content_length
                                                    {
                                                        self.request_queue[i].complete = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let clien_would_delete = Self::route_request(
                        &mut self.request_queue,
                        self.servers.clone(),
                        stream,
                        cookie,
                        &config,
                        &mut poll,
                    );
                    if clien_would_delete {
                        if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
                            eprintln!("Erreur lors de la fermeture du stream: {}", e);
                            Server::error_log(&req, config, "Router::run", file!(), line!(), crate::ServerError::IOError(&e));
                        }
                        self.clients.remove(&event.token());
                    };
                }
            }
        }
    }

    /// Accepte une nouvelle connexion et l'ajoute à la liste des clients.
    fn accept_connection(&mut self, token: Token, poll: &Poll) -> io::Result<()> {
        if let Some(listener) = self.listeners.get_mut(&token) {
            let (mut stream, _) = listener.accept()?;
            if let Err(e) = stream.set_ttl(60) {
                println!("error: timeout - {e}");
            }
            let client_token = Token(self.next_token);
            self.next_token += 1;
            poll.registry()
                .register(&mut stream, client_token, Interest::READABLE)?;
            self.clients.insert(client_token, stream);
        }
        Ok(())
    }

    // Route une requête HTTP et génère une réponse.
    pub fn route_request(
        request_queue: &mut Vec<Request>,
        servers: Vec<Server>,
        stream: &mut TcpStream,
        cookie: String,
        config: &Config,
        poll: &mut Poll,
    ) -> bool {
        // On récupère le hostname, l'adresse ip et le port de la requête
        // On parcoure la liste des serveurs et on vérifie lequel a le hostname, le port et l'ip correspondant
        let mut i = 0;
        while i < request_queue.len() {
            let req = request_queue[i].clone();
            for server in servers.iter() {
                if (server.ip_addr == req.host  || server.hostname == req.host)
                    && server.ports.contains(&req.port)
                {
                    if req.method == "GET" || req.complete {
                        match server.handle_request(stream, req.clone(), cookie.clone(), config) {
                            Ok(_) => {}
                            Err(err) => {
                                match poll.registry().deregister(stream) {
                                    Ok(_) => println!("Client supprimer sur le register d'epoll pour cause d'erreur sur l'ecriture :  {:?}", err),
                                    Err(err) => {
                                        match poll.registry().deregister(stream) {
                                            Ok(_) => println!("Client supprimer sur le register d'epoll en mod ecriture pour cause {:?}",err),
                                            Err(e) => println!("Error while deregising on read stream on read operation: {}", e),
                                        };
                                    return true;
                                    }
                                }
                            }
                        };

                        request_queue.remove(i);
                        if i != 0 {
                            i -= 1;
                        }
                        return false;
                    }
                }
            }
            i += 1;
        }
        false
    }
}
