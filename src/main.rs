use localhost::*;
// Importe le module server (mod.rs)

fn main() -> std::io::Result<()> {
    // Charge le fichier de configuration
    let config = load_config();
    // Crée un routeur et ajoute le serveur
    let mut router = Router::new();

    // Ajouter les serveurs au routeur
    for (_, s) in &config.http.servers {
        router.add_server(s.clone())?;        
    }
    // Démarre le routeur
    router.run(&config)?;

    Ok(())
}
