# Duel Game 🎮

Mini-jeu de duel au tour par tour en Rust, développé dans le cadre du cours de RUST.
Le jeu peut se jouer jusqu'à 4 joueurs. Par défaut, au lancement du programme, seul 2 joueurs pourront jouer.
Pensez à renseigner le pseudonyme des joueurs supplémentaires.

---

## Lancement

```bash
cargo run -- --name1 Amaury --name2 Julien --name3 Joueur3 --name4 Joueur4 --vitality 50 --objectifs 5
```

### Paramètres disponibles

| Paramètre      | Défaut  | Description                                         |
|----------------|---------|-----------------------------------------------------|
| `--name1`      | Amaury  | Nom du joueur 1                                     |
| `--name2`      | Julien  | Nom du joueur 2                                     |
| `--name3`      | None    | Nom du joueur 3 (Pas de joueur par défaut)          |
| `--name4`      | None    | Nom du joueur 4 (Pas de joueur par défaut)          |
| `--vitality`   | 50      | Points de vie initiaux (communs aux deux joueurs)   |
| `--objectifs`  | 5       | Nombre d'objectifs par tour                         |
| `--speed`      | 50      | Délai en ms entre chaque tick du compteur (min : 5) |
| `--strength`   | 50      | Bonus de force ajouté au score brut                 |

---

## Règles du jeu

Le jeu oppose deux jusqu'à quatre joueurs qui s'affrontent en **manches successives**.  
Chaque joueur dispose de quatre caractéristiques : **nom**, **vitalité**, **vitesse** et **force**.
Au lancement du jeu, les joueurs peuvent décider de jouer en mode **Normale** ou en mode **Aléatoire**.
En mode normal, la touche **ENTRÉE** est la seule touche éligible pour figer son score.
En mode aléatoire, une lettre de l'alphabet est choisie aléatoirement pour chaque objectif, elle devient ainsi la touche sur laquelle appuyer pour l'objectif correspondant.

### Déroulement d'un tour

1. Un tableau de N objectifs (nombres entre 0 et 100) est généré aléatoirement.
2. Le joueur appuie sur **ENTRÉE** pour démarrer.
3. Un compteur s'incrémente de 0 à 100 à la vitesse du joueur (en ms par tick).

#### Mode normal
4. Le joueur appuie sur **ENTRÉE** pour figer le compteur sur chaque objectif.

#### Mode aléatoire
4. Le joueur appuie sur **la touche objectif** pour figer le compteur sur chaque objectif.

5. Plus le compteur est proche de l'objectif, plus le score est élevé.

### Calcul du score

Le score de chaque objectif suit cette grille de précision :

| Différence (absolue) | Score de base |
|----------------------|---------------|
| 0                    | 100           |
| 1 – 5                | 80            |
| 6 – 10               | 60            |
| 11 – 20              | 40            |
| 21 – 40              | 20            |
| > 40                 | 0             |

**Formule :** `(score_base + force) / (miss + 1)`

> La variable `miss` s'incrémente à chaque fois que le compteur boucle sur 0.  
> La différence est **circulaire** : un compteur à 95 pour un objectif à 15 donne une différence de 20, pas 80.

Le score de fin de tour est la **moyenne arrondie à l'entier supérieur** des scores de chaque objectif.

### Résolution d'une manche

- Le joueur avec le score le plus élevé **gagne la manche**.
- Le perdant perd en vitalité la **différence entre les deux scores**.
- Le gagnant choisit un **poison** à appliquer au perdant :
  - `1` → −5 de vitesse (compteur plus rapide, plancher : 5 ms)
  - `2` → −5 de force (plancher : 0)

La partie s'arrête quand la **vitalité d'un joueur tombe à zéro ou en dessous**.  
Les joueurs peuvent **relancer une partie** sans relancer le programme.

---

## Exemple de partie

```
##### Démarrage de la partie #####
Joueurs : Amaury vs Julien
Vitalité initiale : 50 | Objectifs par tour : 5

## Manche 1 ##
Au tour de Amaury (Vitality=50, Speed=50, Strength=50)
→ Objectifs : [50, 82, 74, 33, 95]
→ Appuyer sur ENTREE pour démarrer le tour..
→ Objectif  50 : Miss = 1 | Compteur =  36 // Score = (40 + 50) / 2 = 45
→ Objectif  82 : Miss = 0 | Compteur =  80 // Score = (80 + 50) / 1 = 130
→ Objectif  74 : Miss = 0 | Compteur =  70 // Score = (80 + 50) / 1 = 130
→ Objectif  33 : Miss = 1 | Compteur =  43 // Score = (60 + 50) / 2 = 55
→ Objectif  95 : Miss = 1 | Compteur =  90 // Score = (80 + 50) / 2 = 65
# Fin du tour #
→ Score moyen : 85
```

---

## Architecture

| Module        | Rôle                                                       |
|---------------|------------------------------------------------------------|
| `args`        | Parsing des arguments CLI via `clap`                       |
| `player`      | Struct `Player` et ses caractéristiques                    |
| `mechanics`   | Logique pure : diff circulaire, score, génération objets   |
| `counter`     | Compteur temps-réel (threads + atomics)                    |
| `input`       | Lecture stdin (ENTRÉE, choix 1/2)                          |
| `round`       | Orchestration d'un tour, d'une manche, du poison           |

---

## Documentation

```bash
cargo doc --open
```

## Tests

```bash
cargo test
```

## Logs (debug)

```bash
RUST_LOG=debug cargo run
```
