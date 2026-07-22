mod db;
mod etl;
mod queries;

use anyhow::{bail, Context, Result};
use mysql::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Affiche l'aide (usage, commandes disponibles, exemples) sur `stderr`.
///
/// Utilisée à la fois pour `-h`/`--help`, en cas d'absence de commande,
/// et en cas de commande inconnue.
fn print_usage() {
    eprintln!(
        "Usage: adp_trafic_aerien <commande> [options]\n\
         \n\
         Commandes disponibles :\n\
         \x20 init-db   [--schema <fichier.sql>]   Cree le schema (tables, PK, FK)\n\
         \x20 load-data [--data-dir <dossier>]      Charge les 5 CSV dans la base\n\
         \x20 mission1                               Execute les requetes de la Mission 1\n\
         \n\
         Exemples :\n\
         \x20 cargo run --release -- init-db\n\
         \x20 cargo run --release -- load-data --data-dir data\n\
         \x20 cargo run --release -- mission1\n"
    );
}

/// Recherche un flag (ex: `--schema`) dans la liste des arguments CLI et
/// renvoie la valeur qui le suit, convertie en `PathBuf`.
///
/// # Paramètres
/// - `args` : la liste complète des arguments (`env::args().collect()`),
///   `args[0]` étant le nom du binaire.
/// - `flag` : le nom du flag recherché (ex: `"--data-dir"`).
/// - `default` : valeur renvoyée si le flag est absent ou mal formé
///   (présent en dernière position sans valeur qui suit).
///
/// # Exemple
/// Pour `args = ["prog", "load-data", "--data-dir", "mon_dossier"]` et
/// `flag = "--data-dir"`, renvoie `PathBuf::from("mon_dossier")`.
///
/// Implémentation volontairement simple (pas de crate `clap`) : suffisant
/// pour un CLI à options minimales comme celui-ci.
fn arg_value(args: &[String], flag: &str, default: &str) -> PathBuf {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }
    PathBuf::from(default)
}

/// Initialise la base de données à partir d'un fichier de schéma SQL
/// (création des tables, clés primaires, clés étrangères, etc.).
///
/// # Sécurité / séparation des privilèges
/// Utilise une connexion **admin** distincte de la connexion applicative :
/// - `DB_ADMIN_URL` (par défaut `mysql://root@127.0.0.1:3306`), typiquement
///   le compte `root` sans mot de passe, car la création de base/tables
///   nécessite des privilèges élevés (DDL : `CREATE`, `DROP`, etc.).
/// - Le reste de l'application (chargement de données, requêtes) utilise
///   un compte restreint (`adp_user`) configuré via `.env` (voir `db.rs`
///   et le README), qui n'a que les droits nécessaires (DML : `SELECT`,
///   `INSERT`, etc.), pas de droit de modifier le schéma.
///
/// # Nettoyage des commentaires SQL
/// Les commentaires `-- ...` sont retirés **ligne par ligne** avant de
/// découper le script sur les `;`. C'est important : si on cherchait
/// `--` dans le texte complet du fichier d'un seul bloc, un commentaire
/// en début de fichier pouvait supprimer par erreur une vraie instruction
/// SQL placée après lui (bug corrigé : des `DROP`/`CREATE DATABASE`
/// étaient silencieusement ignorés). En traitant chaque ligne
/// indépendamment, un `--` ne peut jamais affecter que le reste de sa
/// propre ligne.
///
/// # Limites connues
/// Le découpage des instructions se fait naïvement sur le caractère `;`.
/// Cela suffit pour un schéma classique (CREATE TABLE, ALTER TABLE...)
/// mais casserait avec des chaînes de caractères contenant `;` ou des
/// blocs `CREATE TRIGGER`/`CREATE PROCEDURE` (qui contiennent leurs
/// propres `;` internes).
///
/// # Erreurs
/// - Connexion admin impossible.
/// - Fichier de schéma introuvable/illisible.
/// - Une instruction SQL échoue : le message d'erreur inclut
///   l'instruction fautive pour faciliter le débogage.
fn init_db(schema_path: &PathBuf) -> Result<()> {
    // Connexion "admin" (root, pas de mot de passe) uniquement utilisée pour créer
    // la base et son schéma. L'application elle-même se connecte ensuite avec un
    // compte restreint (adp_user) défini dans .env - cf db.rs et README.md.
    let url =
        env::var("DB_ADMIN_URL").unwrap_or_else(|_| "mysql://root@127.0.0.1:3306".into());
    let mut conn = mysql::Conn::new(url.as_str()).context("connexion admin MySQL")?;

    let raw = fs::read_to_string(schema_path).context("lecture schema.sql")?;

    // On retire les commentaires ligne par ligne (et pas seulement au niveau du
    // "chunk" complet) avant de découper par ';', sinon un commentaire de tête de
    // fichier peut engloutir la vraie instruction SQL qui le suit sur la même
    // portion de texte (bug corrigé : DROP/CREATE DATABASE étaient ignorés).
    let content: String = raw
        .lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx], // tronque la ligne au début du commentaire
            None => line,              // pas de commentaire -> ligne intacte
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Découpe le script en instructions individuelles et les exécute une
    // par une (pas de transaction globale ici : un schema.sql est en
    // général exécuté une seule fois de façon contrôlée).
    for stmt in content.split(';') {
        let s = stmt.trim();
        if s.is_empty() {
            continue; // ignore les lignes vides / le ';' final
        }
        conn.query_drop(s)
            .with_context(|| format!("Erreur sur l'instruction SQL:\n{s}"))?;
    }
    println!("Base 'trafic_aerien' initialisee avec succes.");
    Ok(())
}

/// Point d'entrée du programme.
///
/// Interface en ligne de commande avec trois sous-commandes :
/// - `init-db   [--schema <fichier.sql>]` : crée le schéma de la base
///   (tables, PK, FK) à partir d'un fichier SQL (`schema.sql` par défaut).
/// - `load-data [--data-dir <dossier>]` : charge les 5 fichiers CSV
///   (`airports`, `airlines`, `planes`, `weather`, `flights`) dans la base
///   via `etl::load_all` (dossier `data` par défaut).
/// - `mission1` : exécute les requêtes d'analyse de la Mission 1
///   (`queries::run_all`).
///
/// Commandes supplémentaires :
/// - `-h` / `--help` : affiche l'aide.
/// - toute autre valeur : affiche l'aide et retourne une erreur.
///
/// # Codes de sortie
/// Grâce au type de retour `Result<()>`, toute erreur (`bail!` ou `?`)
/// fait que le processus se termine avec un code de sortie non nul et
/// affiche l'erreur (comportement standard fourni par `anyhow`/Rust
/// lorsque `main` retourne un `Result`).
fn main() -> Result<()> {
    // Charge les variables d'environnement depuis un éventuel fichier .env
    // (ex : DB_HOST, DB_USER, DB_PASSWORD, DB_ADMIN_URL...).
    let _ = dotenvy::dotenv();
    let args: Vec<String> = env::args().collect();

    // args[0] = nom du binaire, args[1] = commande attendue.
    if args.len() < 2 {
        print_usage();
        bail!("Aucune commande fournie.");
    }

    match args[1].as_str() {
        "init-db" => {
            let schema = arg_value(&args, "--schema", "schema.sql");
            init_db(&schema)?;
        }
        "load-data" => {
            let data_dir = arg_value(&args, "--data-dir", "data");
            let pool = db::build_pool()?;
            etl::load_all(&pool, &data_dir)?;
            println!("Chargement termine.");
        }
        "mission1" => {
            let pool = db::build_pool()?;
            queries::run_all(&pool)?;
        }
        "-h" | "--help" => {
            print_usage();
        }
        other => {
            print_usage();
            bail!("Commande inconnue : {other}");
        }
    }

    Ok(())
}