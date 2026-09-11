# Chess Engine

Un moteur d'échecs écrit **from scratch** en Rust : génération de coups,
détection d'échec/mat, recherche par minimax avec élagage alpha-bêta,
recherche de quiescence, protocole UCI — sans aucune bibliothèque d'échecs
tierce (pas de `shakmaty`, pas de `chess`). Le cœur du moteur (plateau,
génération de coups, évaluation, recherche) n'a **aucune** dépendance
externe ; seul le binaire UCI fait de l'I/O sur `stdin`/`stdout`, en Rust
standard.

## Ce que fait (et ne fait pas) ce moteur

Plateau 8x8 en tableau plein ("mailbox"), FEN import/export complet,
génération de coups légale (pions avec prise en passant et promotion,
roque avec toutes ses conditions, détection de clouage), recherche négamax
avec élagage alpha-bêta et recherche de quiescence, évaluation matériel +
tables pièce-case, approfondissement itératif avec budget de temps,
protocole UCI (utilisable depuis n'importe quelle interface graphique
d'échecs standard).

Ce que ce moteur ne fait **pas** : table de transposition (pas de cache de
positions déjà évaluées — chaque branche de l'arbre de recherche est
recalculée depuis zéro), évaluation par phase de partie (un roi centralisé
est toujours légèrement pénalisé même en finale, où c'est pourtant un
atout), détection de nulle par répétition ou matériel insuffisant, réglage
fin du temps de réflexion (juste un budget "profondeur fixe" ou "date
limite"). Des extensions réelles, mais hors du périmètre pédagogique de ce
projet — voir "Limites connues" plus bas.

## Pourquoi un tableau plein plutôt que des bitboards

La plupart des moteurs de compétition (Stockfish, etc.) représentent le
plateau avec des bitboards : un entier 64 bits par type de pièce/couleur,
où chaque bit correspond à une case, ce qui permet de calculer des
attaques ou des déplacements avec quelques opérations bit à bit au lieu
d'une boucle. C'est nettement plus rapide, mais la logique elle-même
(décalages de bits, masques précalculés) est bien plus opaque à lire et à
déboguer qu'un tableau de 64 cases indexé directement. Pour un projet dont
le but est de comprendre — et prouver par les tests — comment un moteur
fonctionne, la lisibilité l'emporte sur la vitesse brute : `board.rs`,
`movegen.rs` et `eval.rs` se lisent comme du pseudo-code.

## Représentation des coups et `make_move` en copy-make

Un `Move` ne porte que `from`/`to`/`flag` (capture, prise en passant,
roque, promotion) — il ne connaît ni la pièce qui bouge ni ce qu'il
capture, cette information vit dans le plateau au moment de `make_move`.

`make_move` retourne un **nouveau** plateau plutôt que de muter `self` avec
un `unmake` séparé ("copy-make", par opposition à "make/unmake"). Une paire
make/unmake est le choix classique en compétition (pas de réallocation),
mais c'est une source classique de bugs : oublier de restaurer un seul
champ de l'état (droits de roque, horloge de demi-coups, case de prise en
passant...) au unmake casse silencieusement la recherche à une profondeur
donnée seulement, souvent le pire moment pour le remarquer. Un `Board` ne
pèse qu'une centaine d'octets ; le cloner à chaque coup reste largement
assez rapide pour ce projet (perft(5) depuis la position de départ —
4 865 609 feuilles — s'exécute en quelques secondes en mode release) et
élimine toute cette catégorie d'erreurs par construction.

## Légalité : générer large, puis filtrer

La génération de coups se fait en deux passes. `pseudo_legal_moves`
respecte le déplacement de chaque pièce mais peut laisser son propre roi
en échec (un cavalier cloué "pourrait" bouger en pseudo-légal). `legal_moves`
filtre : pour chaque coup pseudo-légal, on **joue** le coup et on vérifie
si le roi du camp qui vient de jouer est attaqué — si oui, le coup est
rejeté.

Détecter directement les coups qui *dévoileraient* un échec (les clouages)
en amont serait plus rapide, mais demanderait de tracer les rayons
d'attaque à travers chaque pièce candidate — et donc bien plus facile de
rater un cas particulier (clouage diagonal vs. clouage de colonne, une
prise en passant qui découvre un clouage horizontal rare mais réel : le roi
et les deux pions concernés alignés sur la même rangée). Ici, la légalité
se réduit à une question unique et déjà testée par ailleurs :
"après ce coup, mon roi est-il attaqué ?" (`Board::is_in_check`, qui
réutilise `is_square_attacked`, la même fonction qui sert à interdire de
roquer à travers une case attaquée). Plus lent, mais correct par
construction.

## Perft : la preuve, pas la confiance

[Perft](https://www.chessprogramming.org/Perft) ("**per**formance **t**est")
compte le nombre de positions atteignables à une profondeur donnée. C'est
la méthode standard de la communauté des programmeurs d'échecs pour
**prouver** qu'un générateur de coups est correct plutôt que de le croire
sur parole, parce que les comptages exacts pour plusieurs positions de
référence sont connus et publiés — dont la position de départ et
["Kiwipete"](https://www.chessprogramming.org/Perft_Results#Position_2),
une position conçue spécifiquement pour piéger les bugs de roque, de prise
en passant et de promotion simultanément. `perft.rs` vérifie ce moteur
contre ces comptages jusqu'à la profondeur 4 (tests rapides, dans la suite
normale) et jusqu'à la profondeur 5 pour la position de départ (test
`#[ignore]`, ~5 millions de feuilles — la CI le lance explicitement en mode
release avec `--ignored`, séparément de la suite rapide).

## Recherche : négamax + alpha-bêta + quiescence

Trois idées assemblées, chacune corrigeant un défaut réel de la
précédente :

- **Négamax** : minimax écrit une seule fois (`-negamax(...)` remplace
  l'alternance explicite max/min) grâce à une évaluation "relative au camp
  au trait" (`eval::evaluate` — un score positif est toujours bon pour
  celui qui doit jouer, quelle que soit sa couleur).
- **Élagage alpha-bêta** : dès qu'une branche prouve qu'elle est trop bonne
  pour être autorisée par l'adversaire (`alpha >= beta`), on arrête de
  l'explorer. Le résultat est identique au minimax complet, seul le nombre
  de nœuds visités change. Le tri des coups (captures d'abord, triées par
  MVV-LVA — *Most Valuable Victim, Least Valuable Attacker*, la capture qui
  gagne le plus avec la pièce la moins chère en premier) est ce qui rend
  cet élagage réellement efficace : couper des branches tôt ne marche que
  si les bons coups sont examinés en premier.
- **Recherche de quiescence** : sans elle, la recherche s'arrête pile à la
  profondeur demandée même si le dernier coup examiné est une capture en
  plein milieu d'un échange — la position semble alors bonne ou mauvaise
  pour une raison purement artificielle (l'"horizon" de la recherche), pas
  parce que l'échange est réellement favorable. La quiescence poursuit la
  recherche au-delà de la profondeur nominale, mais uniquement sur les
  captures, jusqu'à atteindre une position "calme". Le test
  `quiescence_search_avoids_a_losing_queen_trade` construit précisément ce
  piège (une dame peut prendre un pion défendu) et aurait échoué sans cette
  extension.

## Utilisation

```bash
cargo test                                     # 49 tests (44 unitaires + 5 d'intégration)
cargo test -- --ignored --release              # + perft(5) depuis la position de départ (~5M feuilles)
cargo run --bin chess-perft -- 5               # perft manuel depuis la position de départ
cargo run --bin chess-perft -- 4 "<fen>"       # perft manuel depuis une FEN arbitraire
cargo run --bin chess-uci                      # moteur UCI sur stdin/stdout
```

Le binaire `chess-uci` implémente un sous-ensemble du protocole
[UCI](https://en.wikipedia.org/wiki/Universal_Chess_Interface) (`uci`,
`isready`, `ucinewgame`, `position [startpos|fen ...] [moves ...]`,
`go [depth N]`, `quit`) — suffisant pour être piloté par n'importe quelle
interface graphique d'échecs standard qui parle UCI (`cutechess-cli`, par
exemple), ou testé à la main. **`go` est bloquant et `stop` est un
no-op** : voir "Limites connues" ci-dessous avant de brancher une
interface graphique qui compte pouvoir interrompre une recherche en
cours.

```
$ cargo run --bin chess-uci
uci
id name Chess Engine
id author Mahouna
uciok
position startpos moves e2e4 e7e5
go depth 4
info depth 4 score cp 34 nodes 48291
bestmove g1f3
```

## Architecture

```
src/
├── piece.rs      # Color, PieceType, Piece, Square (case indexée 0..64)
├── moves.rs        # Move, MoveFlag (quiet/capture/en passant/roque/promotion)
├── board.rs          # Plateau (tableau de 64 cases), FEN, make_move (copy-make)
├── movegen.rs           # Pseudo-légal → légal, détection d'attaque, roque
├── perft.rs                # Comptage de nœuds, preuve de correction du movegen
├── eval.rs                    # Matériel + tables pièce-case
├── search.rs                     # Négamax + alpha-bêta + quiescence + MVV-LVA
├── uci.rs                           # Protocole UCI (logique pure, testable sans I/O)
└── bin/
    ├── uci.rs                          # Boucle stdin/stdout autour de `uci::UciEngine`
    └── perft.rs                          # Utilitaire perft en ligne de commande
```

## Tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

49 tests, tous verts (44 unitaires + 5 d'intégration en boîte noire). Les
plus significatifs :

- `perft_matches_known_values_for_the_starting_position` /
  `_for_kiwipete` / `_for_a_position_with_en_passant_pins` — la preuve de
  correction du générateur de coups contre des comptages de référence
  publiés, pas une simple vérification "à la main" sur quelques positions.
- `quiescence_search_avoids_a_losing_queen_trade` — le test qui a motivé
  la recherche de quiescence (voir plus haut).
- `search_finds_a_mate_in_one` — vérifie à la fois le bon coup ET que le
  score reflète un mat proche (`MATE_SCORE` ajusté par la profondeur du
  mat, pour préférer un mat en 1 à un mat en 3).
- `a_pinned_piece_cannot_move_and_expose_its_own_king` /
  `castling_is_forbidden_through_an_attacked_square` — cas de légalité
  qu'un générateur de coups naïf oublie fréquemment.
- `scholars_mate_is_detected_as_checkmate` (intégration) — joue
  intégralement le mat du berger depuis la position de départ via l'API
  publique (`Board::make_move` uniquement) et vérifie la position finale.
- `engine_plays_a_short_self_play_game_without_ever_producing_an_illegal_position`
  (intégration) — 30 coups d'auto-jeu, vérifie à chaque demi-coup que le
  camp qui vient de jouer n'est jamais resté en échec.

## Limites connues

- **Pas de table de transposition** : chaque branche de l'arbre de
  recherche est recalculée depuis zéro, même si la même position est
  atteinte par un autre ordre de coups (transposition). Ralentit la
  recherche mais n'affecte pas sa correction.
- **Pas de phase de partie dans l'évaluation** : les tables pièce-case sont
  fixes, un roi centralisé est toujours pénalisé même en finale où c'est un
  atout réel (le roi doit s'activer une fois les dames échangées).
- **Pas de détection de nulle par répétition ou règle des 50 coups
  "officielle"** : la recherche traite en interne 50 demi-coups sans
  capture ni poussée de pion comme un nul immédiat (évite de s'enfoncer
  sans fin dans des lignes stériles), mais ce n'est pas la vraie règle
  (qui doit être *réclamée* par un joueur, pas automatique) — un
  raccourci assumé pour la recherche, pas une implémentation des règles
  de nulle du jeu réel.
- **Pas de gestion fine du temps** : seuls "profondeur fixe" et "date
  limite" sont exposés, pas d'allocation de temps par coup selon le temps
  restant à la pendule (comme le ferait un vrai moteur de tournoi).
- **`go` bloque le thread UCI jusqu'à la fin de la recherche, et `stop` ne
  l'interrompt pas.** `UciEngine::handle_line("go ...")` appelle
  `search::search` de façon synchrone, sans thread séparé ni mécanisme
  d'annulation ; la boucle `stdin`/`stdout` du binaire ne peut donc pas
  lire (ni a fortiori traiter) une commande `stop` tant que la recherche
  en cours n'est pas terminée d'elle-même. Dans le code, `stop` est
  d'ailleurs traité exactement comme `quit` (`vec![]`, aucun effet). Une
  interface graphique standard qui envoie `stop` pour couper une recherche
  trop longue (profondeur élevée, ou limite de temps dépassée côté GUI) ne
  sera donc pas obéie avant la fin naturelle de cette recherche — à garder
  en tête avant de brancher ce moteur sur une GUI en partie cadencée.

## Licence

MIT — voir `LICENSE`.
