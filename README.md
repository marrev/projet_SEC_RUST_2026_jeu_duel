# projet_SEC_RUST_2026_jeu_duel
Développement d’un mini jeu de duel en RUST dans le cadre de la formation SEC
## Groupe : JD, AL

## Présentation : 
Ce jeu se joue au tour par tour entre deux joueurs. Il utilise exclusivement la touche ENTREE du clavier ainsi que le terminal en tant 
qu’ I/O. 

Il oppose deux joueurs disposant de caractéristique propres (nom, vitalité, vitesse, force) susceptibles de changer au cours de la 
partie. Chacune des caractéristiques communes aux joueurs peuvent-être paramétrées au lancement du programme mais doivent 
disposer de valeurs par défaut.

A chaque début de tour, un tableau d’objectif est généré et contient des nombres de 0 à 100 générés aléatoirement. Le joueur doit 
ensuite appuyer sur la touche entrée pour commencer à jouer.

Un compteur s’incrémentant de 0 à 100 doit alors apparaître. Le compteur est propre au joueur, son pas d’incrémentation (en ms) 
correspond à son membre “Vitesse” (de 50ms par défaut). Lorsqu’une interruption est détectée par le programme (via l’appui de la 
touche ENTREE), le compteur se fige, affiche le nombre en cours puis démarre le compteur suivant sur une nouvelle ligne. 

L'objectif du joueur est d'arrêter le compteur au plus près des différents nombres du tableau d'objectif. Le tour du joueur se termine 
lorsqu'une interruption est détectée pour chaque objectif.

## Cahier des charges :
- Les nombres du tableau d’objectifs doivent être générés aléatoirement entre 0 et 100 à chaque tour de jeu.
- Le code doit gérer le fait qu'un résultat de 95 pour un objectif de 15 entraine une difference de 20 points et non de 80.
- Lorsque le compteur arrive à 100, il repasse à 0. Cela incrémente la variable “miss” dont la valeur par défaut est à 0.
- Les valeurs de “miss” et “compteur” doivent être up toutes les 30ms via un thread d’affichage dédié.
- A chaque objectif, le joueur gagne des points en fonction de la différence avec son score :

| Différence (absolue) | 0 | 1 à 5 | 6 à 10 | 11 à 20 | 21 à 40 | > 40 |
|----------------------|---|-------|--------|---------|---------|------|
| Score                | (100 + force) / (miss + 1) | (80 + force) / (miss + 1) | (60 + force) / (miss + 1) | (40 + force) / (miss + 1) | (20 + force) / (miss + 1) | (0 + force) / (miss + 1) |

- Le score final de fin de tour correspond à la moyenne des points sur chacun des objectifs.
- Le score final de fin de tour doit être arrondi à l’entier supérieur.
- A la fin d’une manche :-Le joueur avec le plus de point gagne.-Le perdant perd en vitalité la différence entre les deux scores.-Le gagnant choisi un poison à appliquer au joueur qui impactera ses caractéristiques pour les prochaines 
manches (-5 de Vitesse OU -5 de Force).
- Les caractéristique d’un joueur doivent être affichées au début de son tour de jeu.
- La partie se termine lorsque la vitalité d’un joueur tombe à zéro.
- Les joueurs doivent pouvoir relancer une partie en fin de jeu.
