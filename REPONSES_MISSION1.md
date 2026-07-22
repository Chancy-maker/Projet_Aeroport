# Reponses Mission 1 - Se familiariser avec les donnees

> Toutes les valeurs ci-dessous proviennent d'une **execution reelle** de
> `cargo run --release -- mission1` contre la base MySQL chargee avec les 5
> CSV fournis (1462 aeroports [1458 + 4 correctifs FK], 16 compagnies, 3322
> avions, 3299 releves meteo, 252 704 vols).

## Question 1 - Comptages

| Element | Valeur |
|---|---|
| Aeroports en tout (table `airports`) | **1462** (1458 dans le CSV source + 4 aeroports ajoutes pour corriger les FK manquantes : BQN, PSE, SJU, STT) |
| Aeroports de depart (distincts dans `flights.origin`) | **3** (JFK, LGA, EWR) |
| Aeroports de destination (distincts dans `flights.dest`) | **103** |
| Aeroports sans heure d'ete (`dst = 'N'`) | **23** ✅ correspond a l'indice de l'enonce |
| Fuseaux horaires distincts (`tzone`) | **12** dans la table finale (dont un litteralement `\N` = valeur manquante) — *note : sur le CSV source seul, avant l'ajout des 4 aeroports correctifs, on retombe exactement sur 10, ce qui correspond a l'indice de l'enonce ; l'ajout de BQN/PSE/SJU/STT introduit 2 nouveaux fuseaux (`America/Puerto_Rico`, `America/St_Thomas`)* |
| Compagnies | **16** |
| Avions (table `planes`) | **3322** |
| Vols annules (`dep_time` manquant) | **6481** |

## Question 2 - Aeroport le plus emprunte / top-flop destinations et avions

**Aeroport de depart le plus emprunte : EWR** (91 241 vols).

**Top 10 destinations les plus prisees :**
| Destination | Code | Vols | % du total |
|---|---|---|---|
| Hartsfield Jackson Atlanta Intl | ATL | 12 946 | 5.12% |
| Chicago Ohare Intl | ORD | 12 654 | 5.01% |
| Los Angeles Intl | LAX | 11 895 | 4.71% |
| General Edward Lawrence Logan Intl | BOS | 11 560 | 4.57% |
| Orlando Intl | MCO | 10 637 | 4.21% |
| Charlotte Douglas Intl | CLT | 10 448 | 4.13% |
| San Francisco Intl | SFO | 9 729 | 3.85% |
| Fort Lauderdale Hollywood Intl | FLL | 9 443 | 3.74% |
| Miami Intl | MIA | 8 938 | 3.54% |
| Ronald Reagan Washington Natl | DCA | 7 386 | 2.92% |

**Top 10 destinations les moins prisees :**
| Destination | Code | Vols | % du total |
|---|---|---|---|
| Blue Grass | LEX | 1 | 0.00% |
| South Bend Rgnl | SBN | 4 | 0.00% |
| Montrose Regional Airport | MTJ | 15 | 0.01% |
| Yampa Valley | HDN | 15 | 0.01% |
| Cherry Capital Airport | TVC | 16 | 0.01% |
| Key West Intl | EYW | 17 | 0.01% |
| Palm Springs Intl | PSP | 19 | 0.01% |
| Gallatin Field | BZN | 20 | 0.01% |
| Jackson Hole Airport | JAC | 25 | 0.01% |
| Charlottesville-Albemarle | CHO | 31 | 0.01% |

**Top 10 avions qui ont le plus decolle :** N725MQ (443), N723MQ (394), N713MQ (385),
N722MQ (378), N711MQ (376), N258JB (332), N353JB (316), N542MQ (310), N351JB (310), N228JB (309).

**Top 10 avions qui ont le moins decolle :** un tres grand nombre d'avions n'ont
decolle qu'**une seule fois** (ex. N456UW, N519US, N810AW, N632AW, N913EV,
N945DN, N7AXAA, N826MH, N608SW, N902DA) — l'ordre exact parmi ces ex-aequo est
arbitraire.

## Question 3 - Destinations desservies par compagnie

Classement complet (nb de destinations distinctes) :
ExpressJet (61), Endeavor Air (47), United (45), JetBlue (42), Delta (40),
Envoy Air (20), American (19), Southwest (11), US Airways (5), Virgin
America (5), SkyWest (4), AirTran (3), Mesa (3), Alaska (1), Frontier (1),
Hawaiian (1).

Par compagnie **et** aeroport d'origine : voir le detail complet dans la
sortie de `cargo run --release -- mission1` (section Q3.2) — par exemple 9E
dessert 4 destinations depuis EWR, 34 depuis JFK, 36 depuis LGA.

**Graphiques** : voir `charts/q3_destinations_par_compagnie.png` (barres
horizontales, destinations par compagnie) et
`charts/q3_destinations_par_compagnie_et_origine.png` (barres groupees par
origine).

## Question 4 - Houston et NYC -> Seattle

- Vols ayant atterri a Houston (IAH ou HOU) : **6958**
- Vols NYC -> Seattle (SEA) : **2736 vols**, exploites par **5 compagnies**,
  avec **856 avions "uniques"** (`tailnum` distincts).

## Question 5 - Vols par destination + tri alphabetique

Nombre de vols par destination : requete disponible pour les 103
destinations (extrait des 10 premieres par ordre alphabetique de code dans
`mission1`, ex. ABQ=164, ALB=386, ATL=12946, AUS=1826...).

Le tri complet (destination, origine, compagnie - ordre alphabetique par nom
complet apres jointure) est produit par la requete Q5.2 ; par exemple les
toutes premieres lignes concernent "Akron Canton Regional Airport" depuis "La
Guardia" via "AirTran Airways Corporation".

## Question 6 - Couverture des compagnies

**Compagnies qui n'operent PAS sur les 3 aeroports d'origine (JFK/LGA/EWR) :**
AS (1 seul), F9 (1 seul), FL (1 seul), HA (1 seul), OO (2), VX (2), WN (2), YV (1 seul).
*(Autrement dit : 9E, AA, B6, DL, EV, MQ, UA et US operent bien sur les 3.)*

**Compagnies qui desservent l'ensemble des 103 destinations existantes :**
**aucune** — meme ExpressJet (le plus large reseau) ne couvre que 61
destinations sur 103.

**Tableau origines/destinations par compagnie** : genere via
`GROUP_CONCAT` (voir requete Q6.3), extrait complet disponible en sortie de
`mission1` (ex. AA depuis JFK dessert AUS, BOS, DFW, EGE, FLL, IAH, LAS, LAX,
MCO, MIA, ORD, SAN, SEA, SFO, SJU, STT, TPA).

## Question 7 - Destinations exclusives a une compagnie

**29 destinations** ne sont desservies que par une seule compagnie, par exemple :
- CAK (Akron Canton Regional) -> AirTran uniquement
- ALB (Albany Intl) -> ExpressJet uniquement
- ABQ (Albuquerque) -> JetBlue uniquement
- MDW (Chicago Midway) -> Southwest uniquement
- BZN (Gallatin Field) -> United uniquement

(liste complete des 29 dans la sortie de `mission1`, section Q7).

## Question 8 - Vols United / American / Delta

| Compagnie | Nb de vols |
|---|---|
| American Airlines Inc. (AA) | 24 602 |
| Delta Air Lines Inc. (DL) | 35 975 |
| United Air Lines Inc. (UA) | 44 165 |
| **Total** | **104 742** |

Soit environ **41.4%** de l'ensemble des 252 704 vols du dataset.
