use anyhow::{Context, Result};
use mysql::{Opts, OptsBuilder, Pool};
use std::env;

/// Construit un pool de connexions MySQL à partir des variables d'environnement.
///
/// Variables attendues (voir `.env.example`) :
/// - `DB_HOST`     : adresse du serveur MySQL (optionnelle, défaut = "127.0.0.1")
/// - `DB_PORT`     : port du serveur MySQL (optionnelle, défaut = 3306)
/// - `DB_USER`     : nom d'utilisateur MySQL (**obligatoire**)
/// - `DB_PASSWORD` : mot de passe MySQL (**obligatoire**)
/// - `DB_NAME`     : nom de la base de données (optionnelle, défaut = "trafic_aerien")
///
/// # Sécurité
/// Aucun identifiant n'est codé en dur dans le code source : c'est le point
/// "sécuriser la connexion à la BD" demandé dans l'énoncé (Mission 2).
/// Les secrets doivent être fournis via un fichier `.env` (non versionné,
/// à lister dans `.gitignore`) ou via de vraies variables d'environnement
/// système en production.
///
/// # Erreurs
/// Retourne une erreur si :
/// - `DB_PORT` n'est pas un nombre valide,
/// - `DB_USER` ou `DB_PASSWORD` sont absents de l'environnement,
/// - la création du pool échoue (hôte injoignable, identifiants invalides, etc.).
///
/// # Exemple
/// ```no_run
/// let pool = build_pool()?;
/// let mut conn = pool.get_conn()?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn build_pool() -> Result<Pool> {
    // Charge le fichier .env s'il existe à la racine du projet.
    // On ignore volontairement l'erreur (`let _ = ...`) : en production,
    // les variables sont souvent déjà exportées par le système d'exploitation
    // ou l'orchestrateur (Docker, systemd, CI/CD, etc.), donc l'absence
    // du fichier .env n'est pas un problème.
    let _ = dotenvy::dotenv();

    // --- Hôte du serveur MySQL ---
    // Valeur par défaut "127.0.0.1" (localhost) si non définie.
    let host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    // --- Port du serveur MySQL ---
    // Valeur par défaut "3306" (port standard MySQL), puis conversion en u16.
    // `.context(...)` transforme une erreur de parsing en message clair.
    let port: u16 = env::var("DB_PORT")
        .unwrap_or_else(|_| "3306".to_string())
        .parse()
        .context("DB_PORT invalide")?;

    // --- Identifiants (obligatoires) ---
    // Aucune valeur par défaut ici volontairement : si absents, on échoue
    // explicitement plutôt que de se connecter avec des identifiants vides.
    let user = env::var("DB_USER")
        .context("DB_USER manquant dans l'environnement (.env)")?;
    let password = env::var("DB_PASSWORD")
        .context("DB_PASSWORD manquant dans l'environnement (.env)")?;

    // --- Nom de la base de données ---
    // Valeur par défaut propre au projet si non définie.
    let dbname = env::var("DB_NAME").unwrap_or_else(|_| "trafic_aerien".to_string());

    // Construction des options de connexion via le pattern "builder".
    let opts = OptsBuilder::default()
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(user))
        .pass(Some(password))
        .db_name(Some(dbname));

    // Création effective du pool de connexions.
    // Un pool permet de réutiliser des connexions ouvertes plutôt que
    // d'en recréer une à chaque requête (meilleures performances).
    let pool = Pool::new(Opts::from(opts))
        .context("Impossible de créer le pool MySQL")?;

    Ok(pool)
}
