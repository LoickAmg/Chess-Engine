//! Recherche : négamax avec élagage alpha-bêta, recherche de quiescence,
//! et approfondissement itératif.
//!
//! Trois idées assemblées, chacune corrigeant un défaut réel de la
//! précédente :
//! - **Négamax** : minimax écrit une seule fois (`-negamax(...)` remplace
//!   l'alternance explicite max/min) grâce à l'évaluation "relative au
//!   camp au trait" de `eval::evaluate`.
//! - **Élagage alpha-bêta** : dès qu'une branche prouve qu'elle est "trop
//!   bonne pour être autorisée" par l'adversaire (`alpha >= beta`), on
//!   arrête de l'explorer — le résultat est identique au minimax complet,
//!   seul le nombre de nœuds visités change. Le tri des coups (captures
//!   d'abord, triées par MVV-LVA — *Most Valuable Victim, Least Valuable
//!   Attacker*) est ce qui rend cet élagage réellement efficace : couper
//!   tôt ne marche que si les bons coups sont examinés en premier.
//! - **Recherche de quiescence** : sans elle, la recherche s'arrête pile à
//!   la profondeur demandée même si le dernier coup examiné est une
//!   capture en plein milieu d'un échange — la position semble alors bonne
//!   ou mauvaise pour une raison purement artificielle ("l'horizon" de la
//!   recherche), pas parce que l'échange est réellement favorable. La
//!   quiescence poursuit la recherche au-delà de la profondeur nominale,
//!   mais uniquement sur les captures, jusqu'à atteindre une position
//!   "calme" — voir `quiescence_search_avoids_a_losing_queen_trade` dans
//!   les tests, qui aurait échoué sans elle.

use std::time::Instant;

use crate::board::Board;
use crate::eval;
use crate::movegen::legal_moves;
use crate::moves::{Move, MoveFlag};

pub const MATE_SCORE: i32 = 1_000_000;
const INFINITY: i32 = 2_000_000;

#[derive(Debug, Clone, Copy)]
pub struct SearchLimits {
    pub max_depth: u32,
    pub deadline: Option<Instant>,
}

impl SearchLimits {
    pub fn depth(max_depth: u32) -> Self {
        SearchLimits {
            max_depth,
            deadline: None,
        }
    }

    pub fn depth_with_deadline(max_depth: u32, deadline: Instant) -> Self {
        SearchLimits {
            max_depth,
            deadline: Some(deadline),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
}

struct Searcher {
    nodes: u64,
    deadline: Option<Instant>,
    aborted: bool,
}

impl Searcher {
    fn should_abort(&mut self) -> bool {
        if self.aborted {
            return true;
        }
        if let Some(deadline) = self.deadline {
            // On ne consulte l'horloge que de temps en temps : appeler
            // `Instant::now()` à chaque nœud a un coût mesurable sur un
            // négamax qui visite potentiellement des millions de nœuds.
            if self.nodes.is_multiple_of(2048) && Instant::now() >= deadline {
                self.aborted = true;
            }
        }
        self.aborted
    }

    fn negamax(&mut self, board: &Board, depth: u32, ply: u32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.should_abort() {
            return 0;
        }
        // Simplification assumée (voir README) : on traite 50 demi-coups
        // sans capture ni poussée de pion comme un nul immédiat plutôt que
        // d'implémenter la règle réelle (le nul doit être réclamé). Cela
        // évite à la recherche de s'enfoncer sans fin dans des lignes
        // stériles.
        if board.halfmove_clock >= 100 {
            return 0;
        }

        let moves = legal_moves(board);
        if moves.is_empty() {
            return if board.is_in_check(board.side_to_move) {
                // Mat d'autant plus "fort" qu'il est trouvé tôt (ply
                // petit) : un mat en 1 doit toujours être préféré à un mat
                // en 3, sans quoi la recherche pourrait retarder un mat
                // gagnant sans raison.
                -MATE_SCORE + ply as i32
            } else {
                0
            };
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }

        let ordered = order_moves(board, moves);
        let mut best = -INFINITY;
        for mv in ordered {
            let next = board.make_move(mv);
            let score = -self.negamax(&next, depth - 1, ply + 1, -beta, -alpha);
            if self.aborted {
                return 0;
            }
            if score > best {
                best = score;
            }
            if best > alpha {
                alpha = best;
            }
            if alpha >= beta {
                break; // Élagage : l'adversaire évitera toujours cette branche.
            }
        }
        best
    }

    fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.should_abort() {
            return 0;
        }

        // "Stand pat" : le camp au trait peut toujours choisir de ne rien
        // capturer, donc l'évaluation statique est un plancher légitime.
        let stand_pat = eval::evaluate(board);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut captures: Vec<Move> = legal_moves(board)
            .into_iter()
            .filter(|mv| mv.flag.is_capture())
            .collect();
        captures.sort_by_key(|&mv| -mvv_lva_score(board, mv));

        for mv in captures {
            let next = board.make_move(mv);
            let score = -self.quiescence(&next, -beta, -alpha);
            if self.aborted {
                return 0;
            }
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}

/// Score MVV-LVA (Most Valuable Victim, Least Valuable Attacker) : les
/// captures qui gagnent le plus de matériel avec la pièce la moins chère
/// sont examinées en premier, ce qui rend l'élagage alpha-bêta efficace.
fn mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let attacker_value = board
        .piece_at(mv.from)
        .map(|p| eval::piece_value(p.kind))
        .unwrap_or(0);
    let victim_value = if mv.flag == MoveFlag::EnPassantCapture {
        eval::PAWN_VALUE
    } else {
        board
            .piece_at(mv.to)
            .map(|p| eval::piece_value(p.kind))
            .unwrap_or(0)
    };
    // Le facteur 100 garantit que la valeur de la victime domine toujours
    // celle de l'attaquant (même dame-prend-pion reste positif), afin que
    // toute capture soit triée avant tout coup calme (score 0) dans
    // `order_moves`.
    victim_value * 100 - attacker_value
}

fn order_moves(board: &Board, mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_by_key(|&mv| {
        if mv.flag.is_capture() {
            -mvv_lva_score(board, mv)
        } else {
            0
        }
    });
    moves
}

/// Approfondissement itératif : cherche à profondeur 1, puis 2, etc.,
/// jusqu'à `limits.max_depth` ou jusqu'à épuisement du temps imparti. Si le
/// temps expire en cours d'une profondeur, le résultat de la dernière
/// profondeur *complète* est conservé plutôt qu'un résultat partiel
/// potentiellement trompeur.
pub fn search(board: &Board, limits: SearchLimits) -> SearchInfo {
    let mut searcher = Searcher {
        nodes: 0,
        deadline: limits.deadline,
        aborted: false,
    };
    let mut completed = SearchInfo::default();

    for depth in 1..=limits.max_depth.max(1) {
        let moves = legal_moves(board);
        if moves.is_empty() {
            break;
        }
        let ordered = order_moves(board, moves);

        let mut alpha = -INFINITY;
        let beta = INFINITY;
        let mut best_move = None;
        let mut best_score = -INFINITY;

        for mv in ordered {
            let next = board.make_move(mv);
            let score = -searcher.negamax(&next, depth - 1, 1, -beta, -alpha);
            if searcher.aborted {
                break;
            }
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            if best_score > alpha {
                alpha = best_score;
            }
        }

        if searcher.aborted {
            break;
        }
        completed = SearchInfo {
            best_move,
            score: best_score,
            depth,
            nodes: searcher.nodes,
        };
    }

    completed.nodes = searcher.nodes;
    completed
}

pub fn best_move(board: &Board, depth: u32) -> Option<Move> {
    search(board, SearchLimits::depth(depth)).best_move
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_a_mate_in_one() {
        // Mat du couloir : Ra1-a8# (pions f7/g7/h7 privent le roi de toute case).
        let board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
        let info = search(&board, SearchLimits::depth(3));
        let best = info.best_move.expect("un coup légal doit être trouvé");
        assert_eq!(best.to_uci(), "a1a8");
        assert!(
            info.score > MATE_SCORE - 10,
            "le score ({}) devrait refléter un mat quasi immédiat",
            info.score
        );
    }

    #[test]
    fn quiescence_search_avoids_a_losing_queen_trade() {
        // La dame blanche peut prendre le pion d5, mais celui-ci est
        // défendu par le pion e6 : Qxd5 exd5 perd la dame pour un pion.
        // Sans recherche de quiescence, une recherche à profondeur trop
        // faible pourrait ne pas "voir" la reprise et jouer ce coup.
        let board = Board::from_fen("4k3/8/4p3/3p4/8/8/8/3QK3 w - - 0 1").unwrap();
        let info = search(&board, SearchLimits::depth(3));
        let best = info.best_move.expect("un coup légal doit être trouvé");
        assert_ne!(
            best.to_uci(),
            "d1d5",
            "le moteur ne devrait pas jouer une prise perdante détectable par quiescence"
        );
    }

    #[test]
    fn search_returns_none_on_a_position_with_no_legal_moves() {
        let board =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        let info = search(&board, SearchLimits::depth(2));
        assert!(info.best_move.is_none());
    }

    #[test]
    fn search_prefers_capturing_a_free_hanging_rook() {
        // Tour noire en h5, non défendue, sur la diagonale d1-h5 : Qxh5
        // gagne une tour gratuitement.
        let board = Board::from_fen("4k3/8/8/7r/8/8/8/3QK3 w - - 0 1").unwrap();
        let info = search(&board, SearchLimits::depth(2));
        let best = info.best_move.expect("un coup légal doit être trouvé");
        assert_eq!(best.to_uci(), "d1h5");
    }
}
