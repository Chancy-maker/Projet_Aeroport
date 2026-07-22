use anyhow::Result;
use mysql::prelude::*;
use mysql::Pool;

fn h(title: &str) {
    println!("\n=== {title} ===");
}

/// Question 1 : comptages generaux
pub fn q1_comptages(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;
    h("Q1.1 - Nombre total d'aeroports (table airports)");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM airports")?.unwrap();
    println!("{n}");

    h("Q1.2 - Aeroports de depart distincts (flights.origin)");
    let n: i64 = conn.query_first("SELECT COUNT(DISTINCT origin) FROM flights")?.unwrap();
    println!("{n}");

    h("Q1.3 - Aeroports de destination distincts (flights.dest)");
    let n: i64 = conn.query_first("SELECT COUNT(DISTINCT dest) FROM flights")?.unwrap();
    println!("{n}");

    h("Q1.4 - Aeroports SANS heure d'ete (dst = 'N')");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM airports WHERE dst = 'N'")?.unwrap();
    println!("{n}");

    h("Q1.5 - Nombre de fuseaux horaires distincts (tzone)");
    let n: i64 = conn.query_first("SELECT COUNT(DISTINCT tzone) FROM airports")?.unwrap();
    println!("{n}");
    let list: Vec<String> =
        conn.query("SELECT DISTINCT tzone FROM airports ORDER BY tzone")?;
    println!("{list:?}");

    h("Q1.6 - Nombre de compagnies");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM airlines")?.unwrap();
    println!("{n}");

    h("Q1.7 - Nombre d'avions (table planes)");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM planes")?.unwrap();
    println!("{n}");

    h("Q1.8 - Nombre de vols annules (dep_time IS NULL)");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM flights WHERE dep_time IS NULL")?.unwrap();
    println!("{n}");

    Ok(())
}

/// Question 2 : aeroport le plus emprunte, top/flop destinations, top/flop avions
pub fn q2_top_flop(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;

    h("Q2.1 - Aeroport de depart le plus emprunte");
    let row: (String, i64) = conn
        .query_first("SELECT origin, COUNT(*) c FROM flights GROUP BY origin ORDER BY c DESC LIMIT 1")?
        .unwrap();
    println!("{row:?}");

    h("Q2.2 - Top 10 destinations les plus prisees (nom, nb vols, % du total)");
    let rows: Vec<(String, String, i64, f64)> = conn.query(
        "SELECT a.name, f.dest, COUNT(*) as nb,
                ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM flights), 2) as pct
         FROM flights f JOIN airports a ON f.dest = a.faa
         GROUP BY f.dest, a.name
         ORDER BY nb DESC LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    h("Q2.3 - Top 10 destinations les MOINS prisees (nom, nb vols, % du total)");
    let rows: Vec<(String, String, i64, f64)> = conn.query(
        "SELECT a.name, f.dest, COUNT(*) as nb,
                ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM flights), 2) as pct
         FROM flights f JOIN airports a ON f.dest = a.faa
         GROUP BY f.dest, a.name
         ORDER BY nb ASC LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    h("Q2.4 - Top 10 avions qui ont le plus decolle");
    let rows: Vec<(String, i64)> = conn.query(
        "SELECT tailnum, COUNT(*) c FROM flights WHERE tailnum IS NOT NULL
         GROUP BY tailnum ORDER BY c DESC LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    h("Q2.5 - Top 10 avions qui ont le moins decolle");
    let rows: Vec<(String, i64)> = conn.query(
        "SELECT tailnum, COUNT(*) c FROM flights WHERE tailnum IS NOT NULL
         GROUP BY tailnum ORDER BY c ASC LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    Ok(())
}

/// Question 3 : destinations desservies par compagnie (global + par aeroport d'origine)
pub fn q3_destinations_par_compagnie(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;

    h("Q3.1 - Nb de destinations desservies par compagnie");
    let rows: Vec<(String, String, i64)> = conn.query(
        "SELECT al.carrier, al.name, COUNT(DISTINCT f.dest) as nb_dest
         FROM flights f JOIN airlines al ON f.carrier = al.carrier
         GROUP BY al.carrier, al.name ORDER BY nb_dest DESC",
    )?;
    for r in &rows { println!("{r:?}"); }

    h("Q3.2 - Nb de destinations desservies par compagnie ET par aeroport d'origine");
    let rows2: Vec<(String, String, i64)> = conn.query(
        "SELECT carrier, origin, COUNT(DISTINCT dest) as nb_dest
         FROM flights GROUP BY carrier, origin ORDER BY carrier, origin",
    )?;
    for r in &rows2 { println!("{r:?}"); }

    Ok(())
}

/// Question 4 : Houston + NYC -> Seattle
pub fn q4_houston_seattle(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;

    h("Q4.1 - Nombre de vols ayant atterri a Houston (IAH ou HOU)");
    let n: i64 = conn
        .query_first("SELECT COUNT(*) FROM flights WHERE dest IN ('IAH','HOU')")?
        .unwrap();
    println!("{n}");

    h("Q4.2 - Vols NYC -> Seattle (SEA)");
    let n: i64 = conn.query_first("SELECT COUNT(*) FROM flights WHERE dest = 'SEA'")?.unwrap();
    println!("nb_vols = {n}");
    let n: i64 = conn
        .query_first("SELECT COUNT(DISTINCT carrier) FROM flights WHERE dest = 'SEA'")?
        .unwrap();
    println!("nb_compagnies = {n}");
    let n: i64 = conn
        .query_first("SELECT COUNT(DISTINCT tailnum) FROM flights WHERE dest = 'SEA'")?
        .unwrap();
    println!("nb_avions_uniques = {n}");

    Ok(())
}

/// Question 5 : vols par destination + tri alphabetique avec jointures
pub fn q5_vols_par_destination(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;

    h("Q5.1 - Nombre de vols par destination (echantillon des 10 premieres)");
    let rows: Vec<(String, i64)> = conn.query(
        "SELECT dest, COUNT(*) FROM flights GROUP BY dest ORDER BY dest LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    h("Q5.2 - Vols tries par destination, origine, compagnie (ordre alpha) - echantillon 10 lignes");
    let rows: Vec<(String, String, String)> = conn.query(
        "SELECT ad.name as dest_name, ao.name as origin_name, al.name as carrier_name
         FROM flights f
         JOIN airports ao ON f.origin = ao.faa
         JOIN airports ad ON f.dest = ad.faa
         JOIN airlines al ON f.carrier = al.carrier
         ORDER BY ad.name, ao.name, al.name
         LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    Ok(())
}

/// Question 6 : compagnies partielles / completes
pub fn q6_compagnies_couverture(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;

    h("Q6.1 - Compagnies qui n'operent PAS sur tous les aeroports d'origine (JFK/LGA/EWR)");
    let rows: Vec<(String, i64)> = conn.query(
        "SELECT carrier, COUNT(DISTINCT origin) as nb_origins
         FROM flights GROUP BY carrier HAVING nb_origins < 3",
    )?;
    for r in rows { println!("{r:?}"); }

    h("Q6.2 - Compagnies qui desservent L'ENSEMBLE des destinations existantes");
    let rows: Vec<(String, i64)> = conn.query(
        "SELECT carrier, COUNT(DISTINCT dest) as nb_dest FROM flights
         GROUP BY carrier
         HAVING nb_dest = (SELECT COUNT(DISTINCT dest) FROM flights)",
    )?;
    if rows.is_empty() {
        println!("Aucune compagnie ne dessert l'integralite des destinations.");
    }
    for r in rows { println!("{r:?}"); }

    h("Q6.3 - Tableau origines/destinations par compagnie (echantillon 10 lignes)");
    let rows: Vec<(String, String, String)> = conn.query(
        "SELECT carrier, origin, GROUP_CONCAT(DISTINCT dest ORDER BY dest SEPARATOR ',') as destinations
         FROM flights GROUP BY carrier, origin ORDER BY carrier, origin LIMIT 10",
    )?;
    for r in rows { println!("{r:?}"); }

    Ok(())
}

/// Question 7 : destinations exclusives a une compagnie
pub fn q7_destinations_exclusives(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;
    h("Q7 - Destinations desservies par une seule compagnie (exclusives)");
    let rows: Vec<(String, String, String)> = conn.query(
        "SELECT f.dest, a.name, MIN(al.name) as unique_carrier
         FROM flights f
         JOIN airports a ON f.dest = a.faa
         JOIN airlines al ON f.carrier = al.carrier
         GROUP BY f.dest, a.name
         HAVING COUNT(DISTINCT f.carrier) = 1
         ORDER BY a.name",
    )?;
    println!("Nombre de destinations exclusives : {}", rows.len());
    for r in rows { println!("{r:?}"); }
    Ok(())
}

/// Question 8 : vols United / American / Delta
pub fn q8_united_american_delta(pool: &Pool) -> Result<()> {
    let mut conn = pool.get_conn()?;
    h("Q8 - Nombre de vols exploites par United, American ou Delta");
    let rows: Vec<(String, String, i64)> = conn.query(
        "SELECT al.carrier, al.name, COUNT(*) as nb_vols
         FROM flights f JOIN airlines al ON f.carrier = al.carrier
         WHERE al.name LIKE 'United%' OR al.name LIKE 'American%' OR al.name LIKE 'Delta%'
         GROUP BY al.carrier, al.name",
    )?;
    for r in &rows { println!("{r:?}"); }
    let total: i64 = rows.iter().map(|(_, _, n)| n).sum();
    println!("TOTAL = {total}");
    Ok(())
}

/// Lance l'ensemble des questions de la Mission 1, dans l'ordre.
pub fn run_all(pool: &Pool) -> Result<()> {
    q1_comptages(pool)?;
    q2_top_flop(pool)?;
    q3_destinations_par_compagnie(pool)?;
    q4_houston_seattle(pool)?;
    q5_vols_par_destination(pool)?;
    q6_compagnies_couverture(pool)?;
    q7_destinations_exclusives(pool)?;
    q8_united_american_delta(pool)?;
    Ok(())
}
