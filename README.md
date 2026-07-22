# Projet ADP - Trafic Aerien (Mission 1 & 2)

Analyse du trafic aerien au depart de New York (JFK, LGA, EWR) - donnees BTS.
Stack : **MySQL/MariaDB + Rust**.

## 1. Outils a installer

| Outil | Pourquoi | Lien |
|---|---|---|
| **Rust** (via `rustup`, PAS `apt`) | Compiler/executer le projet | https://rustup.rs |
| **MySQL** ou **MariaDB** (serveur) | Heberger la base | https://dev.mysql.com/downloads/ ou `apt install mariadb-server` / Docker |
| Un client MySQL (CLI `mysql` ou DBeaver/MySQL Workbench) | Inspecter la base a la main | - |
| Git | Partager le repo avec l'equipe | - |

> **Important** : installez Rust via `rustup` et pas via le gestionnaire de paquets de
> votre distribution (`apt`, etc.). Une version de rustc trop ancienne (< 1.77/1.85)
> peut echouer a compiler certaines dependances recentes de l'ecosysteme.
> `rustup` vous donne toujours une version stable a jour.

Verification rapide apres installation :
```bash
rustc --version   # >= 1.80 recommande
cargo --version
mysql --version   # ou mariadb --version
```

## 2. Mise en route

### 2.1 Demarrer MySQL/MariaDB
```bash
# Linux (systemd)
sudo systemctl start mysql        # ou mariadb selon la distro

# Docker (alternative simple, cross-plateforme)
docker run --name adp-mysql -e MYSQL_ROOT_PASSWORD=RootPass2026! -p 3306:3306 -d mysql:8
```

### 2.2 Configurer la connexion
```bash
cp .env.example .env
```
Editez `.env` avec vos identifiants (voir section "Securite" plus bas).

### 2.3 Creer le schema
```bash
cargo run --release -- init-db
```
Cree la base `trafic_aerien`, les 5 tables (`airports`, `airlines`, `planes`,
`weather`, `flights`) avec leurs contraintes PK/FK, et insere les 4 aeroports
manquants (BQN, PSE, SJU, STT) mentionnes dans l'enonce.

> Pensez a creer l'utilisateur applicatif restreint AVANT ou APRES (selon votre
> ordre de preference) avec par exemple :
> ```sql
> CREATE USER 'adp_user'@'%' IDENTIFIED BY 'AdpProjet2026!';   
> GRANT ALL PRIVILEGES ON trafic_aerien.* TO 'adp_user'@'%';
> FLUSH PRIVILEGES;
> ```

### 2.4 Charger les donnees
```bash
cargo run --release -- load-data --data-dir data
```
Charge les 5 CSV (`data/*.csv`) dans les tables, dans le bon ordre (respect des FK).
Prend ~15 secondes pour les 252 704 vols.

### 2.5 Executer les requetes de la Mission 1
```bash
cargo run --release -- mission1
```
Affiche les reponses aux 8 questions de la Mission 1 (voir aussi `REPONSES_MISSION1.md`).

## 3. Structure du projet

```
.
├── Cargo.toml              # dependances Rust
├── schema.sql               # schema SQL complet (tables, PK, FK, contraintes)
├── .env.example              # modele de config de connexion (ne pas committer .env)
├── data/                     # les 5 CSV sources (nettoyes)
│   ├── airports.csv
│   ├── airlines.csv
│   ├── planes.csv
│   ├── weather.csv
│   └── flights.csv
├── charts/                   # graphiques generes pour la question 3
├── src/
│   ├── main.rs               # CLI (init-db / load-data / mission1)
│   ├── db.rs                 # connexion MySQL securisee (via .env)
│   ├── etl.rs                 # chargement des CSV -> MySQL
│   └── queries.rs             # les 8 requetes de la Mission 1
└── REPONSES_MISSION1.md      # reponses commentees, issues d'une execution reelle
```

## 4. Choix de modelisation (Mission 2)

Voir aussi les commentaires en tete de `schema.sql`.

- **`airports.faa`** : PK naturelle (regex `^[A-Z0-9]{3,4}$` verifiee via un `CHECK`).
  4 aeroports references dans `flights` (dest) mais absents du CSV source
  (`BQN`, `PSE`, `SJU`, `STT`) sont ajoutes manuellement dans `schema.sql`.
- **`airlines.carrier`** : PK naturelle (2 caracteres alphanumeriques).
- **`planes.tailnum`** : PK naturelle. **Volontairement, aucune contrainte FK
  stricte** n'est posee entre `flights.tailnum` et `planes.tailnum` : des
  centaines d'avions references dans `flights` n'existent pas dans `planes`
  (American Airlines et Envoy Air rapportent des numeros de flotte plutot que
  des tailnum reels, cf. aide de la table `planes`). Poser une FK stricte
  aurait bloque l'import de ces lignes. Un simple index est cree a la place.
- **`weather`** : PK composite `(year, month, day, hour, origin)`, FK
  `origin -> airports.faa`.
- **`flights`** : PK composite `(year, month, day, hour, carrier, flight)` -
  verifie unique sur les 252 704 lignes reelles (le `flight` seul ne suffit
  pas, cf. enonce). FK vers `airlines` et `airports` (origin/dest). Pas de FK
  sur `tailnum` (voir point precedent).
- La relation many-to-many `airlines <-> airports` n'est pas materialisee par
  une table de jonction dediee : c'est `flights` elle-meme qui joue ce role
  (table de faits).

## 5. Securite de la connexion

- Aucun identifiant en dur dans le code : tout passe par des variables
  d'environnement chargees via le crate `dotenvy` (`src/db.rs`).
- `.env` est ignore par git (`.gitignore`), seul `.env.example` est versionne.
- L'application se connecte avec un compte MySQL **restreint** (`adp_user`,
  droits uniquement sur `trafic_aerien.*`), jamais avec `root` (sauf pour la
  commande `init-db`, qui a besoin de `CREATE/DROP DATABASE`).

## 6. Pour aller plus loin (Mission 3, bonus)

Idees de WebApp de reporting (non incluses ici) :
- Un dashboard Rust (`axum` + `askama`/`sqlx` ou simplement en servant les
  resultats de `queries.rs` en JSON) consomme par un frontend leger.
- Ou une app Shiny (R) / Flask (Python) connectee a la meme base MySQL, en
  suivant les liens fournis dans l'enonce.
