use anyhow::{Context, Result};
use mysql::prelude::*;
use mysql::{params, Pool, TxOpts};
use std::path::Path;

// ============================================================================
// Fonctions utilitaires de conversion CSV -> valeurs SQL
// ============================================================================

/// Convertit une cellule CSV vide (ou ne contenant que des espaces) en `None`
/// (=> `NULL` en SQL), sinon renvoie la valeur "trimée" (espaces retirés).
///
/// Le dataset NYC Flights contient de nombreuses valeurs manquantes
/// (ex : `dep_time` vide pour un vol annulé), qu'il faut représenter par
/// `NULL` plutôt que par une valeur par défaut trompeuse.
fn opt(s: &str) -> Option<&str> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s.trim())
    }
}

/// Convertit une cellule CSV en `Option<i64>`.
///
/// Passe par un parsing en `f64` avant de tronquer en `i64` : cela permet
/// de gérer des valeurs numériques écrites avec une décimale dans le CSV
/// (ex : "120.0" -> 120), ce qui arrive fréquemment dans ce type de export.
/// Renvoie `None` si la cellule est vide ou non numérique.
fn opt_i64(s: &str) -> Option<i64> {
    opt(s).and_then(|v| v.parse::<f64>().ok()).map(|v| v as i64)
}

/// Convertit une cellule CSV en `Option<f64>`.
/// Renvoie `None` si la cellule est vide ou non numérique.
fn opt_f64(s: &str) -> Option<f64> {
    opt(s).and_then(|v| v.parse::<f64>().ok())
}

/// Taille des lots pour l'insertion en masse (table `flights`, ~250k lignes).
/// Un compromis entre :
/// - trop petit -> trop de transactions -> lent (overhead réseau/commit),
/// - trop grand -> risque de dépasser `max_allowed_packet` côté MySQL
///   et transactions trop longues (verrouillage).
const BATCH_SIZE: usize = 2000;

// ============================================================================
// Chargement des tables "de référence" (petites tables, une transaction chacune)
// ============================================================================

/// Charge `airports.csv` (colonnes : faa,name,lat,lon,alt,tz,dst,tzone)
/// dans la table `airports`.
///
/// Utilise `ON DUPLICATE KEY UPDATE` : si un aéroport (clé `faa`) existe déjà,
/// son nom est mis à jour plutôt que de provoquer une erreur de doublon.
/// Cela rend la fonction **idempotente** (rejouable sans effet de bord).
///
/// # Erreurs
/// - Échec d'ouverture/lecture du CSV.
/// - Échec de connexion ou de requête SQL.
///
/// # Retour
/// Le nombre de lignes traitées.
pub fn load_airports(pool: &Pool, path: &Path) -> Result<usize> {
    let mut rdr = csv::Reader::from_path(path).context("lecture airports.csv")?;
    let mut conn = pool.get_conn()?;
    let mut count = 0usize;

    // Une seule transaction pour tout le fichier : soit tout est inséré,
    // soit rien (cohérence en cas d'erreur au milieu du fichier).
    let mut tx = conn.start_transaction(TxOpts::default())?;
    for rec in rdr.records() {
        let r = rec?;
        tx.exec_drop(
            "INSERT INTO airports (faa,name,lat,lon,alt,tz,dst,tzone)
             VALUES (:faa,:name,:lat,:lon,:alt,:tz,:dst,:tzone)
             ON DUPLICATE KEY UPDATE name=VALUES(name)",
            params! {
                "faa" => &r[0],
                "name" => &r[1],
                // NB : unwrap_or(0.0) masque silencieusement une valeur
                // invalide -> à surveiller si les coordonnées sont critiques.
                "lat" => r[2].parse::<f64>().unwrap_or(0.0),
                "lon" => r[3].parse::<f64>().unwrap_or(0.0),
                "alt" => r[4].parse::<i64>().unwrap_or(0),
                "tz" => r[5].parse::<i64>().unwrap_or(0),
                "dst" => &r[6],
                "tzone" => opt(&r[7]),
            },
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}

/// Charge `airlines.csv` (colonnes : carrier,name) dans la table `airlines`.
///
/// Table de référence très simple : code compagnie (`carrier`) -> nom complet.
/// Même logique d'upsert que `load_airports`.
pub fn load_airlines(pool: &Pool, path: &Path) -> Result<usize> {
    let mut rdr = csv::Reader::from_path(path).context("lecture airlines.csv")?;
    let mut conn = pool.get_conn()?;
    let mut count = 0usize;
    let mut tx = conn.start_transaction(TxOpts::default())?;
    for rec in rdr.records() {
        let r = rec?;
        tx.exec_drop(
            "INSERT INTO airlines (carrier,name) VALUES (:carrier,:name)
             ON DUPLICATE KEY UPDATE name=VALUES(name)",
            params! { "carrier" => &r[0], "name" => &r[1] },
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}

/// Charge `planes.csv`
/// (colonnes : tailnum,year,type,manufacturer,model,engines,seats,speed,engine)
/// dans la table `planes`.
///
/// Beaucoup de champs sont optionnels (`opt_i64`) car les fiches avions
/// du dataset source sont souvent incomplètes (année de fabrication
/// inconnue, nombre de sièges non renseigné, etc.).
pub fn load_planes(pool: &Pool, path: &Path) -> Result<usize> {
    let mut rdr = csv::Reader::from_path(path).context("lecture planes.csv")?;
    let mut conn = pool.get_conn()?;
    let mut count = 0usize;
    let mut tx = conn.start_transaction(TxOpts::default())?;
    for rec in rdr.records() {
        let r = rec?;
        tx.exec_drop(
            "INSERT INTO planes (tailnum,year,type,manufacturer,model,engines,seats,speed,engine)
             VALUES (:tailnum,:year,:type,:manufacturer,:model,:engines,:seats,:speed,:engine)
             ON DUPLICATE KEY UPDATE model=VALUES(model)",
            params! {
                "tailnum" => &r[0],
                "year" => opt_i64(&r[1]),
                "type" => &r[2],
                "manufacturer" => &r[3],
                "model" => &r[4],
                "engines" => opt_i64(&r[5]),
                "seats" => opt_i64(&r[6]),
                "speed" => opt_i64(&r[7]),
                "engine" => &r[8],
            },
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}

/// Charge `weather.csv` dans la table `weather`.
/// Colonnes :
/// origin,year,month,day,hour,temp,dewp,humid,wind_dir,wind_speed,
/// wind_gust,precip,pressure,visib,time_hour
///
/// Presque toutes les mesures météo sont optionnelles (capteur en panne,
/// relevé manquant à une heure donnée) -> usage massif de `opt_f64`/`opt_i64`.
///
/// Le champ `time_hour` (format ISO type `2013-01-01T05:00:00Z`) est
/// converti au format compatible MySQL `DATETIME` en remplaçant
/// le `T` par un espace et en supprimant le `Z` final :
/// `"2013-01-01T05:00:00Z"` -> `"2013-01-01 05:00:00"`.
pub fn load_weather(pool: &Pool, path: &Path) -> Result<usize> {
    let mut rdr = csv::Reader::from_path(path).context("lecture weather.csv")?;
    let mut conn = pool.get_conn()?;
    let mut count = 0usize;
    let mut tx = conn.start_transaction(TxOpts::default())?;
    for rec in rdr.records() {
        let r = rec?;
        tx.exec_drop(
            "INSERT INTO weather
             (origin,year,month,day,hour,temp,dewp,humid,wind_dir,wind_speed,wind_gust,precip,pressure,visib,time_hour)
             VALUES (:origin,:year,:month,:day,:hour,:temp,:dewp,:humid,:wind_dir,:wind_speed,:wind_gust,:precip,:pressure,:visib,:time_hour)
             ON DUPLICATE KEY UPDATE temp=VALUES(temp)",
            params! {
                "origin" => &r[0],
                "year" => r[1].parse::<i64>().unwrap_or(0),
                "month" => r[2].parse::<i64>().unwrap_or(0),
                "day" => r[3].parse::<i64>().unwrap_or(0),
                "hour" => r[4].parse::<i64>().unwrap_or(0),
                "temp" => opt_f64(&r[5]),
                "dewp" => opt_f64(&r[6]),
                "humid" => opt_f64(&r[7]),
                "wind_dir" => opt_i64(&r[8]),
                "wind_speed" => opt_f64(&r[9]),
                "wind_gust" => opt_f64(&r[10]),
                "precip" => opt_f64(&r[11]),
                "pressure" => opt_f64(&r[12]),
                "visib" => opt_f64(&r[13]),
                // Conversion du format ISO 8601 vers un DATETIME MySQL.
                "time_hour" => opt(&r[14]).map(|s| s.replace('T', " ").replace('Z', "")),
            },
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}

// ============================================================================
// Chargement de la table volumineuse : flights (insertion par lots)
// ============================================================================

/// Charge `flights.csv`, la table la plus volumineuse (~250 000 lignes),
/// par lots (`batch insert`) plutôt que ligne par ligne, pour des raisons
/// de performance.
///
/// Colonnes :
/// year,month,day,dep_time,sched_dep_time,dep_delay,arr_time,sched_arr_time,
/// arr_delay,carrier,flight,tailnum,origin,dest,air_time,distance,hour,
/// minute,time_hour
///
/// # Stratégie de batch
/// - Les lignes sont accumulées dans un buffer `Vec<mysql::Params>`.
/// - Dès que le buffer atteint `BATCH_SIZE` (2000) lignes, une transaction
///   est ouverte, `exec_batch` insère tout le lot en une seule requête
///   multi-valeurs, puis la transaction est validée (`commit`) et le
///   buffer est vidé (`drain`).
/// - À la fin de la boucle, le reliquat (< `BATCH_SIZE` lignes) est
///   inséré dans un dernier lot.
///
/// # `INSERT IGNORE`
/// Contrairement aux autres tables, on utilise `INSERT IGNORE` plutôt que
/// `ON DUPLICATE KEY UPDATE` : si un vol existe déjà (doublon de clé),
/// la ligne est simplement ignorée sans erreur ni mise à jour. C'est plus
/// rapide et suffisant ici car les vols ne sont pas censés être modifiés
/// après import.
///
/// # Retour
/// Le nombre total de lignes insérées/traitées.
pub fn load_flights(pool: &Pool, path: &Path) -> Result<usize> {
    let mut rdr = csv::Reader::from_path(path).context("lecture flights.csv")?;
    let mut conn = pool.get_conn()?;
    let mut total = 0usize;
    let mut batch: Vec<mysql::Params> = Vec::with_capacity(BATCH_SIZE);

    let insert_sql = "INSERT IGNORE INTO flights
        (year,month,day,dep_time,sched_dep_time,dep_delay,arr_time,sched_arr_time,arr_delay,
         carrier,flight,tailnum,origin,dest,air_time,distance,hour,minute,time_hour)
        VALUES
        (:year,:month,:day,:dep_time,:sched_dep_time,:dep_delay,:arr_time,:sched_arr_time,:arr_delay,
         :carrier,:flight,:tailnum,:origin,:dest,:air_time,:distance,:hour,:minute,:time_hour)";

    for rec in rdr.records() {
        let r = rec?;
        batch.push(
            params! {
                "year" => r[0].parse::<i64>().unwrap_or(0),
                "month" => r[1].parse::<i64>().unwrap_or(0),
                "day" => r[2].parse::<i64>().unwrap_or(0),
                // Horaires/délais/distance : optionnels car un vol annulé
                // n'a pas de dep_time/arr_time réel, par exemple.
                "dep_time" => opt_i64(&r[3]),
                "sched_dep_time" => opt_i64(&r[4]),
                "dep_delay" => opt_i64(&r[5]),
                "arr_time" => opt_i64(&r[6]),
                "sched_arr_time" => opt_i64(&r[7]),
                "arr_delay" => opt_i64(&r[8]),
                "carrier" => &r[9],
                "flight" => r[10].parse::<i64>().unwrap_or(0),
                "tailnum" => opt(&r[11]),
                "origin" => &r[12],
                "dest" => &r[13],
                "air_time" => opt_i64(&r[14]),
                "distance" => opt_i64(&r[15]),
                "hour" => opt_i64(&r[16]),
                "minute" => opt_i64(&r[17]),
                "time_hour" => opt(&r[18]).map(|s| s.replace('T', " ").replace('Z', "")),
            },
        );

        // Dès que le lot est plein, on l'envoie en base et on le vide.
        if batch.len() >= BATCH_SIZE {
            let mut tx = conn.start_transaction(TxOpts::default())?;
            tx.exec_batch(insert_sql, batch.drain(..))?;
            tx.commit()?;
            total += BATCH_SIZE;
        }
    }

    // Dernier lot partiel (reliquat de lignes < BATCH_SIZE).
    if !batch.is_empty() {
        let n = batch.len();
        let mut tx = conn.start_transaction(TxOpts::default())?;
        tx.exec_batch(insert_sql, batch.drain(..))?;
        tx.commit()?;
        total += n;
    }
    Ok(total)
}

// ============================================================================
// Orchestration
// ============================================================================

/// Charge les 5 fichiers CSV dans la base, **dans l'ordre imposé par les
/// contraintes de clé étrangère** :
/// 1. `airports` et `airlines` et `planes` (tables de référence, sans FK
///    sortante) doivent exister avant `flights`, qui référence
///    `origin`/`dest` (-> airports), `carrier` (-> airlines) et
///    `tailnum` (-> planes).
/// 2. `weather` n'a pas de dépendance stricte mais est chargée par
///    cohérence avant `flights`.
///
/// Affiche une progression sur la sortie standard (nombre de lignes
/// chargées par fichier).
///
/// # Erreurs
/// Retourne la première erreur rencontrée (arrête le chargement complet
/// si un des fichiers échoue).
pub fn load_all(pool: &Pool, data_dir: &Path) -> Result<()> {
    println!("Chargement airports.csv ...");
    let n = load_airports(pool, &data_dir.join("airports.csv"))?;
    println!("  -> {n} lignes");

    println!("Chargement airlines.csv ...");
    let n = load_airlines(pool, &data_dir.join("airlines.csv"))?;
    println!("  -> {n} lignes");

    println!("Chargement planes.csv ...");
    let n = load_planes(pool, &data_dir.join("planes.csv"))?;
    println!("  -> {n} lignes");

    println!("Chargement weather.csv ...");
    let n = load_weather(pool, &data_dir.join("weather.csv"))?;
    println!("  -> {n} lignes");

    println!("Chargement flights.csv (peut prendre quelques dizaines de secondes) ...");
    let n = load_flights(pool, &data_dir.join("flights.csv"))?;
    println!("  -> {n} lignes");

    Ok(())
}