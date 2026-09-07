//! Génération de coups : pseudo-légale (respecte les règles de déplacement
//! de chaque pièce mais peut laisser son propre roi en échec) puis légale
//! (filtre la précédente en simulant chaque coup).
//!
//! Séparer les deux passes est un choix classique et volontaire : détecter
//! directement les coups qui *dévoileraient* un échec (les clouages)
//! demanderait de tracer les rayons d'attaque à travers chaque pièce
//! candidate — plus rapide, mais bien plus facile à rater un cas (clouage
//! diagonal vs. clouage de colonne, prise en passant qui découvre un
//! clouage horizontal rare mais réel). Ici, la légalité se réduit à une
//! question unique et déjà testée : "après ce coup, mon roi est-il
//! attaqué ?" (`Board::is_in_check`). Plus lent, mais correct par
//! construction — et `perft.rs` prouve cette correction contre des
//! comptages de référence connus.

use crate::board::Board;
use crate::moves::{Move, MoveFlag};
use crate::piece::{Color, Piece, PieceType, Square};

const ROOK_DIRECTIONS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_OFFSETS: [(i8, i8); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

pub fn is_square_attacked(board: &Board, square: Square, by: Color) -> bool {
    // Pions : un pion de `by` en (f∓1, rank - dir) attaque `square`.
    let dir = by.pawn_direction();
    for df in [-1i8, 1i8] {
        if let Some(sq) = square.try_offset(df, -dir) {
            if board.piece_at(sq) == Some(Piece::new(by, PieceType::Pawn)) {
                return true;
            }
        }
    }

    for &(df, dr) in KNIGHT_OFFSETS.iter() {
        if let Some(sq) = square.try_offset(df, dr) {
            if board.piece_at(sq) == Some(Piece::new(by, PieceType::Knight)) {
                return true;
            }
        }
    }

    for &(df, dr) in KING_OFFSETS.iter() {
        if let Some(sq) = square.try_offset(df, dr) {
            if board.piece_at(sq) == Some(Piece::new(by, PieceType::King)) {
                return true;
            }
        }
    }

    for &(df, dr) in ROOK_DIRECTIONS.iter() {
        if ray_hits(
            board,
            square,
            df,
            dr,
            by,
            &[PieceType::Rook, PieceType::Queen],
        ) {
            return true;
        }
    }
    for &(df, dr) in BISHOP_DIRECTIONS.iter() {
        if ray_hits(
            board,
            square,
            df,
            dr,
            by,
            &[PieceType::Bishop, PieceType::Queen],
        ) {
            return true;
        }
    }

    false
}

fn ray_hits(
    board: &Board,
    from: Square,
    df: i8,
    dr: i8,
    by: Color,
    accepted_kinds: &[PieceType],
) -> bool {
    let mut current = from;
    while let Some(next) = current.try_offset(df, dr) {
        match board.piece_at(next) {
            None => current = next,
            Some(piece) => {
                return piece.color == by && accepted_kinds.contains(&piece.kind);
            }
        }
    }
    false
}

/// Coups pseudo-légaux du camp au trait : respectent le déplacement de
/// chaque pièce mais peuvent laisser leur propre roi en échec.
pub fn pseudo_legal_moves(board: &Board) -> Vec<Move> {
    let color = board.side_to_move;
    let mut moves = Vec::with_capacity(48);

    for (sq, piece) in board.pieces_of(color) {
        match piece.kind {
            PieceType::Pawn => generate_pawn_moves(board, sq, color, &mut moves),
            PieceType::Knight => generate_step_moves(board, sq, color, &KNIGHT_OFFSETS, &mut moves),
            PieceType::Bishop => {
                generate_sliding_moves(board, sq, color, &BISHOP_DIRECTIONS, &mut moves)
            }
            PieceType::Rook => {
                generate_sliding_moves(board, sq, color, &ROOK_DIRECTIONS, &mut moves)
            }
            PieceType::Queen => {
                generate_sliding_moves(board, sq, color, &ROOK_DIRECTIONS, &mut moves);
                generate_sliding_moves(board, sq, color, &BISHOP_DIRECTIONS, &mut moves);
            }
            PieceType::King => {
                generate_step_moves(board, sq, color, &KING_OFFSETS, &mut moves);
                generate_castling_moves(board, color, &mut moves);
            }
        }
    }

    moves
}

/// Coups réellement légaux : ceux des coups pseudo-légaux qui, une fois
/// joués, ne laissent pas le roi du camp qui vient de jouer en échec.
pub fn legal_moves(board: &Board) -> Vec<Move> {
    let mover = board.side_to_move;
    pseudo_legal_moves(board)
        .into_iter()
        .filter(|&mv| !board.make_move(mv).is_in_check(mover))
        .collect()
}

fn generate_step_moves(
    board: &Board,
    from: Square,
    color: Color,
    offsets: &[(i8, i8)],
    moves: &mut Vec<Move>,
) {
    for &(df, dr) in offsets {
        if let Some(to) = from.try_offset(df, dr) {
            match board.piece_at(to) {
                None => moves.push(Move::new(from, to, MoveFlag::Quiet)),
                Some(target) if target.color != color => {
                    moves.push(Move::new(from, to, MoveFlag::Capture))
                }
                Some(_) => {}
            }
        }
    }
}

fn generate_sliding_moves(
    board: &Board,
    from: Square,
    color: Color,
    directions: &[(i8, i8)],
    moves: &mut Vec<Move>,
) {
    for &(df, dr) in directions {
        let mut current = from;
        while let Some(to) = current.try_offset(df, dr) {
            match board.piece_at(to) {
                None => {
                    moves.push(Move::new(from, to, MoveFlag::Quiet));
                    current = to;
                }
                Some(target) => {
                    if target.color != color {
                        moves.push(Move::new(from, to, MoveFlag::Capture));
                    }
                    break;
                }
            }
        }
    }
}

const PROMOTION_KINDS: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

fn generate_pawn_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let dir = color.pawn_direction();
    let promotion_rank = color.promotion_rank();

    // Poussée simple + double, uniquement sur cases vides.
    if let Some(one_ahead) = from.try_offset(0, dir) {
        if board.piece_at(one_ahead).is_none() {
            push_pawn_move(from, one_ahead, promotion_rank, MoveFlag::Quiet, moves);

            let start_rank = match color {
                Color::White => 1,
                Color::Black => 6,
            };
            if from.rank() == start_rank {
                if let Some(two_ahead) = from.try_offset(0, 2 * dir) {
                    if board.piece_at(two_ahead).is_none() {
                        moves.push(Move::new(from, two_ahead, MoveFlag::DoublePawnPush));
                    }
                }
            }
        }
    }

    // Captures diagonales (normales et en passant).
    for df in [-1i8, 1i8] {
        let Some(target_sq) = from.try_offset(df, dir) else {
            continue;
        };
        match board.piece_at(target_sq) {
            Some(target) if target.color != color => {
                push_pawn_move(from, target_sq, promotion_rank, MoveFlag::Capture, moves);
            }
            None if board.en_passant == Some(target_sq) => {
                moves.push(Move::new(from, target_sq, MoveFlag::EnPassantCapture));
            }
            _ => {}
        }
    }
}

fn push_pawn_move(
    from: Square,
    to: Square,
    promotion_rank: u8,
    base_flag: MoveFlag,
    moves: &mut Vec<Move>,
) {
    if to.rank() == promotion_rank {
        for &kind in PROMOTION_KINDS.iter() {
            let flag = if base_flag.is_capture() {
                MoveFlag::PromotionCapture(kind)
            } else {
                MoveFlag::Promotion(kind)
            };
            moves.push(Move::new(from, to, flag));
        }
    } else {
        moves.push(Move::new(from, to, base_flag));
    }
}

fn generate_castling_moves(board: &Board, color: Color, moves: &mut Vec<Move>) {
    let rank = color.home_rank();
    let king_start = Square::new(4, rank);
    if board.piece_at(king_start) != Some(Piece::new(color, PieceType::King)) {
        return; // Le roi n'est plus sur sa case d'origine : pas de roque possible.
    }
    let opponent = color.opposite();
    if board.is_square_attacked(king_start, opponent) {
        return; // On ne roque jamais hors d'un échec.
    }

    let (has_kingside, has_queenside) = match color {
        Color::White => (
            board.castling.white_kingside,
            board.castling.white_queenside,
        ),
        Color::Black => (
            board.castling.black_kingside,
            board.castling.black_queenside,
        ),
    };

    if has_kingside {
        let f = Square::new(5, rank);
        let g = Square::new(6, rank);
        if board.piece_at(f).is_none()
            && board.piece_at(g).is_none()
            && !board.is_square_attacked(f, opponent)
            && !board.is_square_attacked(g, opponent)
        {
            moves.push(Move::new(king_start, g, MoveFlag::CastleKingside));
        }
    }

    if has_queenside {
        let d = Square::new(3, rank);
        let c = Square::new(2, rank);
        let b = Square::new(1, rank);
        if board.piece_at(d).is_none()
            && board.piece_at(c).is_none()
            && board.piece_at(b).is_none()
            && !board.is_square_attacked(d, opponent)
            && !board.is_square_attacked(c, opponent)
        {
            moves.push(Move::new(king_start, c, MoveFlag::CastleQueenside));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_has_exactly_twenty_legal_moves() {
        let board = Board::starting_position();
        // 16 poussées de pions (8 simples + 8 doubles) + 4 sauts de cavalier.
        assert_eq!(legal_moves(&board).len(), 20);
    }

    #[test]
    fn knight_in_the_corner_has_exactly_two_moves() {
        let board = Board::from_fen("k7/8/8/8/8/8/8/N6K w - - 0 1").unwrap();
        let knight_moves: Vec<_> = legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == Square::from_algebraic("a1").unwrap())
            .collect();
        assert_eq!(knight_moves.len(), 2);
    }

    #[test]
    fn rook_sliding_is_blocked_by_own_piece_and_captures_enemy_piece() {
        let board = Board::from_fen("8/8/8/3p4/3R4/3P4/8/k6K w - - 0 1").unwrap();
        let rook_moves: Vec<_> = legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == Square::from_algebraic("d4").unwrap())
            .collect();
        // Vers le bas : bloquée par son propre pion en d3, ne peut pas avancer.
        assert!(!rook_moves
            .iter()
            .any(|mv| mv.to == Square::from_algebraic("d3").unwrap()));
        // Vers le haut : peut avancer jusqu'à d5 et capturer le pion noir.
        assert!(rook_moves
            .iter()
            .any(|mv| mv.to == Square::from_algebraic("d5").unwrap() && mv.flag.is_capture()));
        // Ne peut pas sauter par-dessus la capture pour continuer en d6.
        assert!(!rook_moves
            .iter()
            .any(|mv| mv.to == Square::from_algebraic("d6").unwrap()));
    }

    #[test]
    fn pawn_promotion_generates_all_four_piece_choices() {
        let board = Board::from_fen("8/4P3/8/8/8/8/8/k6K w - - 0 1").unwrap();
        let promotions: Vec<_> = legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == Square::from_algebraic("e7").unwrap())
            .collect();
        assert_eq!(promotions.len(), 4);
        for kind in [
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
        ] {
            assert!(promotions
                .iter()
                .any(|mv| mv.flag == MoveFlag::Promotion(kind)));
        }
    }

    #[test]
    fn en_passant_capture_is_only_available_right_after_the_double_push() {
        let board = Board::from_fen("8/8/8/3pP3/8/8/8/k6K w - d6 0 1").unwrap();
        let moves = legal_moves(&board);
        assert!(moves.iter().any(|mv| mv.flag == MoveFlag::EnPassantCapture));

        // Sans la case en passant enregistrée (un coup plus tard), le
        // même coup ne doit plus être proposé.
        let board_no_ep = Board::from_fen("8/8/8/3pP3/8/8/8/k6K w - - 0 1").unwrap();
        let moves_no_ep = legal_moves(&board_no_ep);
        assert!(!moves_no_ep
            .iter()
            .any(|mv| mv.flag == MoveFlag::EnPassantCapture));
    }

    #[test]
    fn castling_is_forbidden_while_in_check() {
        // Tour noire sur la colonne e : le roi blanc est en échec, ne peut pas roquer.
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2r w KQkq - 0 1").unwrap();
        assert!(board.is_in_check(Color::White));
        let moves = legal_moves(&board);
        assert!(!moves
            .iter()
            .any(|mv| mv.flag == MoveFlag::CastleKingside || mv.flag == MoveFlag::CastleQueenside));
    }

    #[test]
    fn castling_is_forbidden_through_an_attacked_square() {
        // Tour noire sur la colonne f : la case de passage du petit roque est attaquée.
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .unwrap()
            .make_move(Move::new(
                Square::from_algebraic("h8").unwrap(),
                Square::from_algebraic("f8").unwrap(),
                MoveFlag::Quiet,
            ));
        let moves = legal_moves(&board);
        assert!(!moves.iter().any(|mv| mv.flag == MoveFlag::CastleKingside));
        // Le grand roque reste disponible, lui n'est pas concerné.
        assert!(moves.iter().any(|mv| mv.flag == MoveFlag::CastleQueenside));
    }

    #[test]
    fn a_pinned_piece_cannot_move_and_expose_its_own_king() {
        // Tour noire en h4, roi blanc en e1, tour blanche clouée en e4 sur la colonne e...
        // Configuration plus simple : cavalier blanc clouable sur la colonne e.
        let board = Board::from_fen("4r3/8/8/8/4N3/8/8/4K3 w - - 0 1").unwrap();
        let knight_moves: Vec<_> = legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == Square::from_algebraic("e4").unwrap())
            .collect();
        assert!(
            knight_moves.is_empty(),
            "un cavalier cloué sur la colonne e ne devrait avoir aucun coup légal"
        );
    }

    #[test]
    fn king_cannot_move_next_to_the_opposing_king() {
        // Rois séparés d'une case (e3 vs e5, avec e4 vide entre eux) : deux
        // rois adjacents dès le départ serait une position illégale en soi,
        // donc pas un bon point de départ pour ce test.
        let board = Board::from_fen("8/8/8/4k3/8/4K3/8/8 w - - 0 1").unwrap();
        let king_moves: Vec<_> = legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == Square::from_algebraic("e3").unwrap())
            .collect();
        // d4, e4, f4 sont toutes adjacentes au roi noir en e5 : interdites.
        for forbidden in ["d4", "e4", "f4"] {
            assert!(!king_moves
                .iter()
                .any(|mv| mv.to == Square::from_algebraic(forbidden).unwrap()));
        }
        // d3 et f3, elles, restent légales.
        for allowed in ["d3", "f3"] {
            assert!(king_moves
                .iter()
                .any(|mv| mv.to == Square::from_algebraic(allowed).unwrap()));
        }
    }

    #[test]
    fn checkmate_position_has_zero_legal_moves() {
        // Position finale du "mat du berger inversé" / fool's mate :
        // 1.f3 e5 2.g4 Qh4# — la dame noire mat le roi blanc via la
        // diagonale h4-e1, entièrement dégagée par 1.f3.
        let board =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        assert!(board.is_in_check(Color::White));
        assert!(legal_moves(&board).is_empty());
    }
}
