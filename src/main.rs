use localhost::*;
// Importe le module server (mod.rs)

fn main() -> std::io::Result<()> {
    // Charge le fichier de configuration
    let mut config = load_config();

    let check_config = verify_config(&mut config);
    if check_config.is_err() {
        return check_config;
    }
    
    // Crée un routeur et ajoute le serveur
    let mut router = Router::new();

    println!("---------------------------------------");
    println!("Liste des adresses disponibles");
    println!("---------------------------------------");
    // Ajouter les serveurs au routeur
    for (_, s) in &config.http.servers {
        router.add_server(s.clone())?;        
    }
    println!("---------------------------------------");
    // Démarre le routeur
    router.run(&config)?;

    Ok(())
}
