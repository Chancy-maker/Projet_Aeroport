-- =========================================================
-- Projet ADP - Trafic Aerien
-- Schema MySQL / MariaDB
-- =========================================================
-- Design decisions (voir README.md section "Choix de modelisation"):
--
-- 1) airports.faa est PK naturelle (nomenclature respectee : 3-4 car. alphanum.)
--    -> 4 aeroports absents de la table source mais references dans flights
--       (BQN, PSE, SJU, STT) sont ajoutes manuellement (cf INSERT plus bas).
--
-- 2) planes.tailnum est PK naturelle, mais des centaines d'avions references
--    dans flights n'existent pas dans planes (AA et MQ reportent des numeros
--    de flotte, pas des tailnum -> non appariables). On NE MET PAS de
--    contrainte FK stricte flights.tailnum -> planes.tailnum pour ne pas
--    bloquer l'import (perte de donnees sinon). On garde un index simple
--    et on documente l'anomalie (question 2, Mission 2).
--
-- 3) PK composite de flights = (year, month, day, hour, carrier, flight)
--    (flight seul n'est pas unique, cf enonce).
--
-- 4) PK composite de weather = (year, month, day, hour, origin).
--
-- 5) many-to-many entre airlines et airports est materialise via flights
--    (pas de table de jonction dediee, flights EST la table de faits).
-- =========================================================

DROP DATABASE IF EXISTS trafic_aerien;
CREATE DATABASE trafic_aerien CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE trafic_aerien;

-- ---------------------------------------------------------
-- Table: airports
-- ---------------------------------------------------------
CREATE TABLE airports (
    faa     VARCHAR(4)      NOT NULL,
    name    VARCHAR(255)    NOT NULL,
    lat     DECIMAL(10,7)   NOT NULL,
    lon     DECIMAL(10,7)   NOT NULL,
    alt     INT             NOT NULL,
    tz      SMALLINT        NOT NULL,
    dst     CHAR(1)         NOT NULL,   -- A / U / N
    tzone   VARCHAR(64)     NULL,
    PRIMARY KEY (faa),
    CONSTRAINT chk_airports_faa CHECK (faa REGEXP '^[A-Z0-9]{3,4}$'),
    CONSTRAINT chk_airports_dst CHECK (dst IN ('A','U','N'))
) ENGINE=InnoDB;

-- ---------------------------------------------------------
-- Table: airlines
-- ---------------------------------------------------------
CREATE TABLE airlines (
    carrier VARCHAR(2)   NOT NULL,
    name    VARCHAR(255) NOT NULL,
    PRIMARY KEY (carrier),
    CONSTRAINT chk_airlines_carrier CHECK (carrier REGEXP '^[A-Z0-9]{2}$')
) ENGINE=InnoDB;

-- ---------------------------------------------------------
-- Table: planes
-- ---------------------------------------------------------
CREATE TABLE planes (
    tailnum      VARCHAR(10)  NOT NULL,
    year         SMALLINT     NULL,
    type         VARCHAR(100) NULL,
    manufacturer VARCHAR(100) NULL,
    model        VARCHAR(100) NULL,
    engines      SMALLINT     NULL,
    seats        SMALLINT     NULL,
    speed        SMALLINT     NULL,
    engine       VARCHAR(100) NULL,
    PRIMARY KEY (tailnum),
    CONSTRAINT chk_planes_tailnum CHECK (tailnum REGEXP '^N[A-Z0-9]+$')
) ENGINE=InnoDB;

-- ---------------------------------------------------------
-- Table: weather
-- ---------------------------------------------------------
CREATE TABLE weather (
    origin      VARCHAR(4)     NOT NULL,
    year        SMALLINT       NOT NULL,
    month       TINYINT        NOT NULL,
    day         TINYINT        NOT NULL,
    hour        TINYINT        NOT NULL,
    temp        DECIMAL(5,2)   NULL,
    dewp        DECIMAL(5,2)   NULL,
    humid       DECIMAL(5,2)   NULL,
    wind_dir    SMALLINT       NULL,
    wind_speed  DECIMAL(8,4)   NULL,
    wind_gust   DECIMAL(12,8)  NULL,
    precip      DECIMAL(6,3)   NULL,
    pressure    DECIMAL(6,1)   NULL,
    visib       DECIMAL(5,2)   NULL,
    time_hour   DATETIME       NULL,
    PRIMARY KEY (year, month, day, hour, origin),
    CONSTRAINT fk_weather_origin FOREIGN KEY (origin) REFERENCES airports(faa)
) ENGINE=InnoDB;

-- ---------------------------------------------------------
-- Table: flights (table de faits)
-- ---------------------------------------------------------
CREATE TABLE flights (
    year            SMALLINT     NOT NULL,
    month           TINYINT      NOT NULL,
    day             TINYINT      NOT NULL,
    dep_time        SMALLINT     NULL,
    sched_dep_time  SMALLINT     NULL,
    dep_delay       INT          NULL,
    arr_time        SMALLINT     NULL,
    sched_arr_time  SMALLINT     NULL,
    arr_delay       INT          NULL,
    carrier         VARCHAR(2)   NOT NULL,
    flight          INT          NOT NULL,
    tailnum         VARCHAR(10)  NULL,
    origin          VARCHAR(4)   NOT NULL,
    dest            VARCHAR(4)   NOT NULL,
    air_time        INT          NULL,
    distance        INT          NULL,
    hour            TINYINT      NOT NULL,
    minute          TINYINT      NULL,
    time_hour       DATETIME     NULL,
    PRIMARY KEY (year, month, day, hour, carrier, flight),
    KEY idx_flights_tailnum (tailnum),
    KEY idx_flights_origin (origin),
    KEY idx_flights_dest (dest),
    KEY idx_flights_carrier (carrier),
    CONSTRAINT fk_flights_carrier FOREIGN KEY (carrier) REFERENCES airlines(carrier),
    CONSTRAINT fk_flights_origin  FOREIGN KEY (origin)  REFERENCES airports(faa),
    CONSTRAINT fk_flights_dest    FOREIGN KEY (dest)    REFERENCES airports(faa)
    -- Pas de FK sur tailnum -> planes : voir note en tete de fichier (point 2)
) ENGINE=InnoDB;

-- ---------------------------------------------------------
-- Correctif FK : 4 aeroports references dans flights (dest/origin)
-- mais absents de la table source airports.csv
-- ---------------------------------------------------------
INSERT INTO airports (faa, name, lat, lon, alt, tz, dst, tzone) VALUES
('BQN', 'Rafael Hernandez Airport', 18.4949, -67.1294, 237, -4, 'A', 'America/Puerto_Rico'),
('PSE', 'Mercedita Airport',        18.0106, -66.5631, 29,  -4, 'A', 'America/Puerto_Rico'),
('SJU', 'San Juan Airport',         18.4394, -66.0018, 9,   -4, 'A', 'America/Puerto_Rico'),
('STT', 'Cyril E. King Airport',    18.3373, -64.9734, 21,  -4, 'A', 'America/St_Thomas');
