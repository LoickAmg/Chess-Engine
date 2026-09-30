//! Le « professeur » : niveaux de jeu, et analyse pédagogique d'un coup (qualité,
//! meilleur coup, et explications en français compréhensibles par un débutant).

use std::time::{Duration, Instant};

use crate::board::Board;
use crate::eval::piece_value;
use crate::movegen::legal_moves;
use crate::moves::{Move, MoveFlag};
use crate::piece::{Color, PieceType, Square};
use crate::san::{to_san, Language};
use crate::search::{search, SearchLimits, MATE_SCORE};

/// Nom d'une pièce en français, avec son article (« la dame », « le cavalier »).
pub fn piece_name(kind: PieceType) -> &'static str {
    match kind {
        PieceType::Pawn => "le pion",
        PieceType::Knight => "le cavalier",
        PieceType::Bishop => "le fou",
        PieceType::Rook => "la tour",
        PieceType::Queen => "la dame",
        PieceType::King => "le roi",
    }
}

fn piece_name_bare(kind: PieceType) -> &'static str {
    piece_name(kind).split(' ').nth(1).unwrap_or("")
}

/// Petit générateur pseudo-aléatoire (xorshift) : pas de dépendance, et reproductible
/// dans les tests en fixant la graine.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

pub struct Level {
    pub depth: u32,
    /// Écart toléré (centipions) avec le meilleur coup : le moteur choisit au hasard
    /// parmi les coups « assez bons », ce qui rend les petits niveaux humains et battables.
    pub margin: i32,
    pub think_ms: u64,
}

pub fn level(n: u8) -> Level {
    match n {
        0 | 1 => Level {
            depth: 1,
            margin: 400,
            think_ms: 300,
        },
        2 => Level {
            depth: 2,
            margin: 150,
            think_ms: 500,
        },
        3 => Level {
            depth: 3,
            margin: 60,
            think_ms: 900,
        },
        4 => Level {
            depth: 4,
            margin: 15,
            think_ms: 1500,
        },
        _ => Level {
            depth: 5,
            margin: 0,
            think_ms: 3000,
        },
    }
}

/// Score (du point de vue du camp au trait) de chaque coup légal, à profondeur `depth`.
fn score_moves(board: &Board, depth: u32, deadline: Instant) -> Vec<(Move, i32)> {
    legal_moves(board)
        .into_iter()
        .map(|mv| {
            let next = board.make_move(mv);
            let info = search(
                &next,
                SearchLimits::depth_with_deadline(depth.saturating_sub(1).max(1), deadline),
            );
            let score = if legal_moves(&next).is_empty() {
                if next.is_in_check(next.side_to_move) {
                    MATE_SCORE
                } else {
                    0
                }
            } else {
                -info.score
            };
            (mv, score)
        })
        .collect()
}

/// Coup joué par le moteur au niveau `level_n` (1 = débutant … 5 = maître).
pub fn engine_move(board: &Board, level_n: u8, seed: u64) -> Option<Move> {
    let lv = level(level_n);
    let deadline = Instant::now() + Duration::from_millis(lv.think_ms);
    if lv.margin == 0 {
        return search(board, SearchLimits::depth_with_deadline(lv.depth, deadline)).best_move;
    }
    let scored = score_moves(board, lv.depth, deadline);
    let best = scored.iter().map(|(_, s)| *s).max()?;
    // Un mat trouvé n'est jamais laissé passer, même aux petits niveaux.
    if best > MATE_SCORE - 100 {
        return scored.into_iter().find(|(_, s)| *s == best).map(|(m, _)| m);
    }
    let candidates: Vec<Move> = scored
        .into_iter()
        .filter(|(_, s)| best - s <= lv.margin)
        .map(|(m, _)| m)
        .collect();
    let mut rng = Rng(seed | 1);
    candidates.get(rng.below(candidates.len())).copied()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl Quality {
    pub fn label(self) -> &'static str {
        match self {
            Quality::Best => "Meilleur coup",
            Quality::Excellent => "Excellent",
            Quality::Good => "Bon coup",
            Quality::Inaccuracy => "Imprécision",
            Quality::Mistake => "Erreur",
            Quality::Blunder => "Gaffe",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Quality::Best => "best",
            Quality::Excellent => "excellent",
            Quality::Good => "good",
            Quality::Inaccuracy => "inaccuracy",
            Quality::Mistake => "mistake",
            Quality::Blunder => "blunder",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveReview {
    pub quality: Quality,
    /// Perte en centipions par rapport au meilleur coup (≥ 0).
    pub loss: i32,
    pub best_move: Option<Move>,
    pub best_san: Option<String>,
    /// Évaluation après le coup, en centipions, du point de vue des Blancs.
    pub eval_after_white: i32,
    pub messages: Vec<String>,
}

fn mate_distance(score: i32) -> Option<i32> {
    // Score `MATE_SCORE` = mat immédiat ; chaque paire de demi-coups en plus retire 2.
    (score.abs() > MATE_SCORE - 1000).then(|| (MATE_SCORE - score.abs()) / 2 + 1)
}

/// Pièces de `color` attaquées et pas (assez) défendues : en prise.
pub fn hanging_pieces(board: &Board, color: Color) -> Vec<(Square, PieceType)> {
    let enemy = color.opposite();
    board
        .pieces_of(color)
        .filter(|(_, p)| p.kind != PieceType::King)
        .filter(|(sq, p)| {
            if !board.is_square_attacked(*sq, enemy) {
                return false;
            }
            if !board.is_square_attacked(*sq, color) {
                return true;
            }
            // Défendue, mais attaquée par une pièce de moindre valeur : perte quand même.
            cheapest_attacker(board, *sq, enemy).is_some_and(|v| v < piece_value(p.kind))
        })
        .map(|(sq, p)| (sq, p.kind))
        .collect()
}

fn cheapest_attacker(board: &Board, target: Square, by: Color) -> Option<i32> {
    // On simule un camp `by` au trait pour lister ses captures sur `target`.
    let mut probe = board.clone();
    probe.side_to_move = by;
    probe.en_passant = None;
    legal_moves(&probe)
        .into_iter()
        .filter(|m| m.to == target && m.flag.is_capture())
        .filter_map(|m| probe.piece_at(m.from).map(|p| piece_value(p.kind)))
        .min()
}

fn has_mate_in_one(board: &Board) -> Option<Move> {
    legal_moves(board).into_iter().find(|&m| {
        let next = board.make_move(m);
        next.is_in_check(next.side_to_move) && legal_moves(&next).is_empty()
    })
}

/// Analyse le coup `mv` (légal) joué dans `board`, avec une recherche de profondeur `depth`.
pub fn review_move(board: &Board, mv: Move, depth: u32) -> MoveReview {
    let me = board.side_to_move;
    let deadline = Instant::now() + Duration::from_millis(2500);
    let scored = score_moves(board, depth, deadline);
    let (best_move, best_score) = scored
        .iter()
        .max_by_key(|(_, s)| *s)
        .map(|(m, s)| (Some(*m), *s))
        .unwrap_or((None, 0));
    let played_score = scored
        .iter()
        .find(|(m, _)| *m == mv)
        .map(|(_, s)| *s)
        .unwrap_or(best_score);
    let loss = (best_score - played_score).max(0);
    let after = board.make_move(mv);

    let quality = if Some(mv) == best_move || loss <= 5 {
        Quality::Best
    } else if loss <= 30 {
        Quality::Excellent
    } else if loss <= 80 {
        Quality::Good
    } else if loss <= 160 {
        Quality::Inaccuracy
    } else if loss <= 320 {
        Quality::Mistake
    } else {
        Quality::Blunder
    };

    let mut messages = Vec::new();
    let moving = board
        .piece_at(mv.from)
        .map(|p| p.kind)
        .unwrap_or(PieceType::Pawn);

    // 1. Ce que le coup accomplit.
    if after.is_in_check(after.side_to_move) && legal_moves(&after).is_empty() {
        messages.push(
            "Échec et mat ! Le roi adverse est attaqué et n'a plus aucune case pour s'échapper."
                .into(),
        );
    } else if after.is_in_check(after.side_to_move) {
        messages.push("Échec : le roi adverse est attaqué, il doit réagir immédiatement.".into());
    }
    if mv.flag.is_capture() {
        let victim = if mv.flag == MoveFlag::EnPassantCapture {
            Some(PieceType::Pawn)
        } else {
            board.piece_at(mv.to).map(|p| p.kind)
        };
        if let Some(v) = victim {
            messages.push(format!(
                "Tu captures {} ({} point{}).",
                piece_name(v),
                piece_value(v) / 100,
                if piece_value(v) >= 200 { "s" } else { "" }
            ));
        }
        if mv.flag == MoveFlag::EnPassantCapture {
            messages.push("C'était une prise en passant, le coup spécial des pions !".into());
        }
    }
    if matches!(
        mv.flag,
        MoveFlag::CastleKingside | MoveFlag::CastleQueenside
    ) {
        messages.push("Bien joué : le roque met ton roi à l'abri et active ta tour.".into());
    }
    if let Some(p) = mv.flag.promotion() {
        messages.push(format!("Promotion : ton pion devient {} !", piece_name(p)));
    }

    // 2. Principes d'ouverture (les 10 premiers coups).
    if board.fullmove_number <= 10 {
        let back_rank = if me == Color::White { 0 } else { 7 };
        if matches!(moving, PieceType::Knight | PieceType::Bishop) && mv.from.rank() == back_rank {
            messages.push(format!(
                "Bon principe : tu développes ton {}.",
                piece_name_bare(moving)
            ));
        }
        if moving == PieceType::Pawn
            && matches!(mv.to.to_algebraic().as_str(), "e4" | "d4" | "e5" | "d5")
        {
            messages.push("Bon principe : tu occupes le centre avec un pion.".into());
        }
        if moving == PieceType::Queen && board.fullmove_number <= 5 && !mv.flag.is_capture() {
            messages.push(
                "Attention : sortir la dame trop tôt l'expose aux attaques des pièces adverses."
                    .into(),
            );
        }
        if moving == PieceType::King
            && !matches!(
                mv.flag,
                MoveFlag::CastleKingside | MoveFlag::CastleQueenside
            )
        {
            messages.push("En début de partie, déplacer le roi (sans roquer) le laisse exposé et t'empêche de roquer.".into());
        }
    }

    // 3. Ce qui ne va pas.
    if let Some(Some(n)) = (best_score > MATE_SCORE - 1000).then(|| mate_distance(best_score)) {
        if played_score < best_score {
            let best_txt = best_move
                .map(|b| to_san(board, b, Language::French))
                .unwrap_or_default();
            messages.push(format!("Tu avais un mat en {n} : {best_txt} !"));
        }
    }
    let hanging = hanging_pieces(&after, me);
    if let Some((sq, kind)) = hanging.iter().max_by_key(|(_, k)| piece_value(*k)) {
        if quality >= Quality::Inaccuracy || piece_value(*kind) >= 300 {
            messages.push(format!(
                "Attention : {} en {} peut être capturé{} par l'adversaire.",
                piece_name(*kind),
                sq.to_algebraic(),
                if matches!(kind, PieceType::Rook | PieceType::Queen) {
                    "e"
                } else {
                    ""
                }
            ));
        }
    }
    if has_mate_in_one(&after).is_some() {
        messages.push(
            "Danger : l'adversaire peut maintenant te mettre échec et mat en un coup !".into(),
        );
    }
    if quality >= Quality::Mistake {
        if let Some(b) = best_move {
            if b.flag.is_capture() {
                if let Some(v) = board.piece_at(b.to) {
                    messages.push(format!(
                        "Tu pouvais gagner {} avec {}.",
                        piece_name(v.kind),
                        to_san(board, b, Language::French)
                    ));
                }
            }
        }
    }

    let eval_after_white = {
        let reply = search(
            &after,
            SearchLimits::depth_with_deadline(depth.saturating_sub(1).max(1), deadline),
        );
        let s = if legal_moves(&after).is_empty() {
            if after.is_in_check(after.side_to_move) {
                -MATE_SCORE
            } else {
                0
            }
        } else {
            reply.score
        };
        // `s` est du point de vue du camp au trait après le coup (l'adversaire).
        if after.side_to_move == Color::White {
            s
        } else {
            -s
        }
    };

    MoveReview {
        quality,
        loss,
        best_san: best_move.map(|b| to_san(board, b, Language::French)),
        best_move,
        eval_after_white,
        messages,
    }
}

impl PartialOrd for Quality {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Quality {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::san::parse_uci;

    fn review(fen: &str, uci: &str) -> MoveReview {
        let board = Board::from_fen(fen).unwrap();
        review_move(&board, parse_uci(&board, uci).unwrap(), 3)
    }

    #[test]
    fn hanging_the_queen_is_a_blunder_with_an_explanation() {
        // La dame blanche va en h5 où le cavalier f6 la prend.
        let r = review(
            "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
            "d1h5",
        );
        assert_eq!(r.quality, Quality::Blunder, "{r:?}");
        assert!(
            r.messages.iter().any(|m| m.contains("la dame en h5")),
            "{:?}",
            r.messages
        );
    }

    #[test]
    fn missing_a_mate_in_one_is_pointed_out() {
        let r = review("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", "a1a2");
        assert!(
            r.messages.iter().any(|m| m.contains("mat en 1")),
            "{:?}",
            r.messages
        );
        assert_eq!(r.best_san.as_deref(), Some("Ta8#"));
    }

    #[test]
    fn the_mating_move_is_the_best_move() {
        let r = review("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", "a1a8");
        assert_eq!(r.quality, Quality::Best);
        assert!(r.messages.iter().any(|m| m.contains("Échec et mat")));
    }

    #[test]
    fn opening_principles_are_praised() {
        let r = review(crate::board::STARTING_FEN, "e2e4");
        assert!(r.quality <= Quality::Good, "{r:?}");
        assert!(r.messages.iter().any(|m| m.contains("centre")));
    }

    #[test]
    fn engine_levels_always_return_a_legal_move_and_take_free_mates() {
        let board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
        for lv in 1..=5 {
            let mv = engine_move(&board, lv, 42).expect("un coup");
            assert_eq!(mv.to_uci(), "a1a8", "niveau {lv}");
        }
        let start = Board::starting_position();
        for seed in 1..20 {
            assert!(legal_moves(&start).contains(&engine_move(&start, 1, seed).unwrap()));
        }
    }
}
