# LOCALHOST

[Lien vers UML](https://drive.google.com/file/d/1NsJBzeaeA0gZZAz5MwWCea9PH7dJX5mO/view?usp=sharing)


# Implémentation d'un Serveur Web en Rust

Ce projet implémente un serveur web personnalisé en Rust avec support pour la gestion de fichiers, la gestion des sessions et l'exécution CGI.

## Vue d'ensemble

Le serveur est construit avec une architecture modulaire qui gère :
- Servir des fichiers statiques
- Lister les répertoires
- Télécharger des fichiers
- Créer des dossiers
- Gérer les sessions
- Exécuter des scripts CGI (Ruby)
- Gérer les erreurs
- Journaliser les accès et les erreurs

## Composants principaux

### 1. Module Serveur (`server(mod).rs`)
Le cœur du serveur qui :
- Traite les requêtes HTTP
- Gère les opérations sur les fichiers
- Implémente la gestion des sessions
- Traite les scripts CGI
- Gère les réponses d'erreur

Exemple de fonctionnalités :
```rust
// Gestion des requêtes HTTP
pub fn handle_request(&self, stream: &mut TcpStream, request: Request) {
    // Traitement des requêtes GET, POST, DELETE
    // Gestion des fichiers et dossiers
    // Réponses aux clients
}
```

### 2. Module Routeur (`router.rs`)
Responsable de :
- Gérer les connexions TCP
- Router les requêtes vers les bons gestionnaires
- Gérer plusieurs instances de serveur
- Gérer les connexions clients

Le routeur utilise un système de tokens pour identifier chaque connexion :
```rust
pub struct Router {
    servers: Vec<Server>,
    sessions: HashMap<Token, Session>,
    listeners: HashMap<Token, TcpListener>,
    clients: HashMap<Token, TcpStream>
}
```

### 3. Module Session (`session.rs`)
Gère :
- Les sessions utilisateur
- Les cookies
- L'expiration des sessions

Chaque session contient :
```rust
pub struct Session {
    id: String,               // Identifiant unique
    validity_time: DateTime,  // Durée de validité
    expiration_time: i64     // Temps d'expiration
}
```

### 4. Module Requête/Réponse
S'occupe de :
- Parser les requêtes HTTP
- Formater les réponses HTTP
- Gérer les données de formulaire multipart
- Gérer les téléchargements de fichiers

## Configuration

Le serveur utilise un fichier de configuration TOML (`config.toml`) qui définit :

```toml
[log_files]
error_log = "chemin/vers/error.log"
access_log = "chemin/vers/access.log"
events_limit = 1024

[http]
access_log_format = "..."
timeout = 60
size_limit = 1048576

[http.servers.server1]
ip_addr = "127.0.0.1"
hostname = "localhost"
ports = [8080, 8081]
root_directory = "public"
```

## Fonctionnalités de sécurité

1. **Gestion des Sessions**
   - Identifiants de session uniques générés de manière sécurisée
   - Timeouts configurables
   - Gestion sécurisée des cookies avec encryption

2. **Opérations sur les Fichiers**
   - Validation des téléchargements
   - Prévention de la traversée de répertoire
   - Restrictions sur les types de fichiers

3. **Gestion des Erreurs**
   - Journalisation détaillée
   - Pages d'erreur personnalisées
   - Réponses d'erreur sécurisées

## Système de Journalisation

Le serveur implémente deux types de logs :

1. **Log d'Accès**
   - Enregistre toutes les requêtes HTTP
   - Format : `timestamp méthode status taille_envoyée`
   - Format configurable

2. **Log d'Erreurs**
   - Enregistre les erreurs serveur
   - Inclut les traces de pile
   - Facilite le débogage

## Support CGI

Le serveur supporte l'exécution de scripts Ruby :
- Exécute les fichiers `.rb`
- Gère la sortie des scripts
- Gère les erreurs d'exécution

## Gestion des Fichiers

1. **Téléchargement**
   - Gestion du multipart
   - Limites de taille
   - Restrictions de type

2. **Opérations sur les Répertoires**
   - Création de dossiers
   - Listage de contenu
   - Suppression de fichiers/dossiers

3. **Interface de Listage**
   - UI moderne avec icônes
   - Détection des types
   - Options de tri et filtrage

## Interface Utilisateur

Le serveur inclut plusieurs templates HTML :
- Interface de listage des répertoires
- Pages d'erreur
- Formulaires d'upload
- Modales de confirmation

## Performance

1. **Gestion des Connexions**
   - E/S non bloquantes
   - Architecture événementielle
   - Pool de connexions

2. **Gestion des Ressources**
   - Gestion efficace de la mémoire
   - Timeouts de connexion
   - Nettoyage des ressources

## Tests

Pour tester le serveur :
1. Démarrer : `cargo run`
2. Accéder via : `http://localhost:8080`
3. Tester les opérations fichiers
4. Surveiller les logs

## Recommandations de Sécurité

1. **Configuration**
   - Permissions appropriées
   - Limites d'upload
   - Méthodes autorisées

2. **Surveillance**
   - Revue régulière des logs
   - Monitoring des ressources
   - Audit de sécurité

3. **Maintenance**
   - Mises à jour régulières
   - Correctifs de sécurité
   - Rotation des logs

## Limitations 

- Traitement mono-thread
- Support MIME limité
- Implémentation CGI basique
- Pas de support SSL/TLS

## Structure du Code

```
src/
├── main.rs           # Point d'entrée
├── mod.rs         # Implémentation du serveur
├── router.rs         # Gestion du routage
├── session.rs        # Gestion des sessions
├── request.rs        # Traitement des requêtes
├── response.rs       # Formatage des réponses
└── templates/        # Templates HTML
```

## Notes pour les Auditeurs

1. **Points d'Attention**
   - La gestion des sessions dans `session.rs`
   - La validation des fichiers dans `server.rs`
   - Le traitement des requêtes dans `request.rs`

2. **Bonnes Pratiques**
   - Validation des entrées
   - Gestion des erreurs robuste
   - Journalisation complète

3. **Sécurité**
   - Vérification des chemins
   - Validation des types MIME
   