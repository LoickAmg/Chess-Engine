//! Représentation du plateau : tableau de 64 cases ("mailbox"), état de la
//! partie (trait, droits de roque, prise en passant, horloges), et
//! FEN import/export.
//!
//! Le choix d'un tableau plein plutôt que des bitboards est délibéré pour
//! ce projet pédagogique : chaque case s'indexe directement (`squares[sq]`),
//! la génération de coups et la détection d'attaque se lisent comme du
//! pseudo-code, au prix d'une performance moindre qu'un moteur de
//! compétition — un compromis assumé (voir le README).
//!
//! `make_move` fonctionne en **copy-make** : il retourne un *nouveau*
//! plateau plutôt que de muter `self` avec un `unmake` séparé. Une paire
//! make/unmake est le choix classique pour la performance (pas de
//! réallocation), mais c'est aussi une source classique de bugs si un seul
//! champ de l'état (droits de roque, horloge de demi-coups...) est oublié
//! au unmake. Un plateau ne pèse qu'une centaine d'octets ; le cloner à
//! chaque coup reste largement assez rapide pour ce projet et élimine toute
//! une catégorie d'erreurs par construction.

use crate::moves::{Move, MoveFlag};
use crate::piece::{Color, Piece, PieceType, Square};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn none() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    fn to_fen(self) -> String {
        let mut s = String::new();
        if self.white_kingside {
            s.push('K');
        }
        if self.white_queenside {
            s.push('Q');
        }
        if self.black_kingside {
            s.push('k');
        }
        if self.black_queenside {
            s.push('q');
        }
        if s.is_empty() {
            s.push('-');
        }
        s
    }

    fn from_fen(field: &str) -> Self {
        if field == "-" {
            return CastlingRights::none();
        }
        CastlingRights {
            white_kingside: field.contains('K'),
            white_queenside: field.contains('Q'),
            black_kingside: field.contains('k'),
            black_queenside: field.contains('q'),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::none(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn starting_position() -> Board {
        Board::from_fen(STARTING_FEN).expect("la FEN de départ est valide par construction")
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.squares[sq.index()]
    }

    pub fn set_piece_at(&mut self, sq: Square, piece: Option<Piece>) {
        self.squares[sq.index()] = piece;
    }

    pub fn king_square(&self, color: Color) -> Option<Square> {
        (0..64)
            .map(Square::from_index_u8)
            .find(|&sq| self.piece_at(sq) == Some(Piece::new(color, PieceType::King)))
    }

    pub fn pieces_of(&self, color: Color) -> impl Iterator<Item = (Square, Piece)> + '_ {
        (0..64).map(Square::from_index_u8).filter_map(move |sq| {
            self.piece_at(sq)
                .filter(|p| p.color == color)
                .map(|p| (sq, p))
        })
    }

    pub fn is_square_attacked(&self, square: Square, by: Color) -> bool {
        crate::movegen::is_square_attacked(self, square, by)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        match self.king_square(color) {
            Some(king_sq) => self.is_square_attacked(king_sq, color.opposite()),
            // Un plateau sans roi ne devrait jamais arriver en jeu réel,
            // mais certaines positions de test (perft partiels, etc.)
            // peuvent en construire un délibérément — on ne panique pas.
            None => false,
        }
    }

    /// Applique `mv` et retourne le plateau résultant. Ne vérifie PAS la
    /// légalité (roi laissé en échec) — c'est la responsabilité de
    /// `movegen::legal_moves`, qui filtre après coup. `make_move` fait
    /// confiance à son appelant sur la validité pseudo-légale du coup.
    pub fn make_move(&self, mv: Move) -> Board {
        let mut next = self.clone();
        let moving_piece = self
            .piece_at(mv.from)
            .expect("make_move: aucune pièce sur la case de départ");
        let mover_color = moving_piece.color;

        next.en_passant = None;

        let is_pawn_move = moving_piece.kind == PieceType::Pawn;
        let is_capture = mv.flag.is_capture();

        // Prise en passant : le pion capturé n'est pas sur la case
        // d'arrivée mais à côté de la case de départ.
        if mv.flag == MoveFlag::EnPassantCapture {
            let captured_sq = Square::new(mv.to.file(), mv.from.rank());
            next.set_piece_at(captured_sq, None);
        }

        next.set_piece_at(mv.from, None);
        let placed = match mv.flag.promotion() {
            Some(promo_kind) => Piece::new(mover_color, promo_kind),
            None => moving_piece,
        };
        next.set_piece_at(mv.to, Some(placed));

        // Roque : déplacer aussi la tour concernée.
        match mv.flag {
            MoveFlag::CastleKingside => {
                let rank = mv.from.rank();
                let rook_from = Square::new(7, rank);
                let rook_to = Square::new(5, rank);
                let rook = next.piece_at(rook_from);
                next.set_piece_at(rook_from, None);
                next.set_piece_at(rook_to, rook);
            }
            MoveFlag::CastleQueenside => {
                let rank = mv.from.rank();
                let rook_from = Square::new(0, rank);
                let rook_to = Square::new(3, rank);
                let rook = next.piece_at(rook_from);
                next.set_piece_at(rook_from, None);
                next.set_piece_at(rook_to, rook);
            }
            _ => {}
        }

        // Case de prise en passant ouverte par une poussée double de pion.
        if mv.flag == MoveFlag::DoublePawnPush {
            let mid_rank = (mv.from.rank() + mv.to.rank()) / 2;
            next.en_passant = Some(Square::new(mv.from.file(), mid_rank));
        }

        // Mise à jour des droits de roque : un roi ou une tour qui bouge,
        // ou une tour capturée sur sa case d'origine, retire le droit
        // correspondant — dans les deux sens (peu importe qui a bougé la
        // pièce visée, une tour disparue ne peut plus roquer).
        next.update_castling_rights_after_move(mv.from, mv.to);

        next.side_to_move = mover_color.opposite();
        next.halfmove_clock = if is_pawn_move || is_capture {
            0
        } else {
            self.halfmove_clock + 1
        };
        if mover_color == Color::Black {
            next.fullmove_number += 1;
        }

        next
    }

    fn update_castling_rights_after_move(&mut self, from: Square, to: Square) {
        for sq in [from, to] {
            match sq {
                s if s == Square::new(4, 0) => {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                }
                s if s == Square::new(4, 7) => {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
                s if s == Square::new(7, 0) => self.castling.white_kingside = false,
                s if s == Square::new(0, 0) => self.castling.white_queenside = false,
                s if s == Square::new(7, 7) => self.castling.black_kingside = false,
                s if s == Square::new(0, 7) => self.castling.black_queenside = false,
                _ => {}
            }
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(format!("FEN incomplète : '{fen}'"));
        }

        let mut board = Board::empty();

        let ranks: Vec<&str> = fields[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(format!(
                "FEN invalide : {} rangées au lieu de 8",
                ranks.len()
            ));
        }
        // La FEN liste les rangées de la 8 vers la 1.
        for (rank_from_top, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_from_top as u8;
            let mut file: u8 = 0;
            for c in rank_str.chars() {
                if let Some(empty_count) = c.to_digit(10) {
                    file += empty_count as u8;
                } else {
                    let piece = Piece::from_fen_char(c)
                        .ok_or_else(|| format!("caractère de pièce invalide : '{c}'"))?;
                    if file >= 8 {
                        return Err(format!("rangée FEN trop longue : '{rank_str}'"));
                    }
                    board.set_piece_at(Square::new(file, rank), Some(piece));
                    file += 1;
                }
            }
            if file != 8 {
                return Err(format!("rangée FEN incomplète : '{rank_str}'"));
            }
        }

        board.side_to_move = match fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("trait invalide : '{other}'")),
        };

        board.castling = CastlingRights::from_fen(fields[2]);

        board.en_passant = match fields[3] {
            "-" => None,
            algebraic => Some(
                Square::from_algebraic(algebraic)
                    .ok_or_else(|| format!("case de prise en passant invalide : '{algebraic}'"))?,
            ),
        };

        board.halfmove_clock = fields.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        board.fullmove_number = fields.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut ranks = Vec::with_capacity(8);
        for rank_from_top in 0..8u8 {
            let rank = 7 - rank_from_top;
            let mut row = String::new();
            let mut empty_run = 0u8;
            for file in 0..8u8 {
                match self.piece_at(Square::new(file, rank)) {
                    Some(piece) => {
                        if empty_run > 0 {
                            row.push_str(&empty_run.to_string());
                            empty_run = 0;
                        }
                        row.push(piece.to_fen_char());
                    }
                    None => empty_run += 1,
                }
            }
            if empty_run > 0 {
                row.push_str(&empty_run.to_string());
            }
            ranks.push(row);
        }
        let placement = ranks.join("/");
        let side = match self.side_to_move {
            Color::White => "w",
            Color::Black => "b",
        };
        let castling = self.castling.to_fen();
        let en_passant = self
            .en_passant
            .map(|sq| sq.to_algebraic())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "{placement} {side} {castling} {en_passant} {} {}",
            self.halfmove_clock, self.fullmove_number
        )
    }
}

impl Square {
    // Petit alias interne pour éviter la friction de type dans les
    // itérateurs `(0..64).map(...)` ci-dessus (`Square::new` prend deux
    // arguments file/rank, pas un seul index brut).
    fn from_index_u8(i: u8) -> Square {
        Square::from_index(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_has_expected_piece_count_and_side_to_move() {
        let board = Board::starting_position();
        assert_eq!(board.side_to_move, Color::White);
        let white_pieces = board.pieces_of(Color::White).count();
        let black_pieces = board.pieces_of(Color::Black).count();
        assert_eq!(white_pieces, 16);
        assert_eq!(black_pieces, 16);
        assert_eq!(
            board.piece_at(Square::from_algebraic("e1").unwrap()),
            Some(Piece::new(Color::White, PieceType::King))
        );
        assert_eq!(
            board.piece_at(Square::from_algebraic("e8").unwrap()),
            Some(Piece::new(Color::Black, PieceType::King))
        );
    }

    #[test]
    fn fen_round_trip_for_starting_position() {
        let board = Board::starting_position();
        assert_eq!(board.to_fen(), STARTING_FEN);
    }

    #[test]
    fn fen_round_trip_for_a_midgame_position() {
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn from_fen_rejects_a_short_rank() {
        let bad = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert!(Board::from_fen(bad).is_err());
    }

    #[test]
    fn double_pawn_push_opens_en_passant_square_behind_the_pawn() {
        let board = Board::starting_position();
        let mv = Move::new(
            Square::from_algebraic("e2").unwrap(),
            Square::from_algebraic("e4").unwrap(),
            MoveFlag::DoublePawnPush,
        );
        let next = board.make_move(mv);
        assert_eq!(next.en_passant, Square::from_algebraic("e3"));
    }

    #[test]
    fn moving_the_king_forfeits_both_castling_rights() {
        let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let mv = Move::new(
            Square::from_algebraic("e1").unwrap(),
            Square::from_algebraic("e2").unwrap(),
            MoveFlag::Quiet,
        );
        board = board.make_move(mv);
        assert!(!board.castling.white_kingside);
        assert!(!board.castling.white_queenside);
        assert!(board.castling.black_kingside);
        assert!(board.castling.black_queenside);
    }

    #[test]
    fn castling_kingside_also_moves_the_rook() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let mv = Move::new(
            Square::from_algebraic("e1").unwrap(),
            Square::from_algebraic("g1").unwrap(),
            MoveFlag::CastleKingside,
        );
        let next = board.make_move(mv);
        assert_eq!(
            next.piece_at(Square::from_algebraic("g1").unwrap()),
            Some(Piece::new(Color::White, PieceType::King))
        );
        assert_eq!(
            next.piece_at(Square::from_algebraic("f1").unwrap()),
            Some(Piece::new(Color::White, PieceType::Rook))
        );
        assert_eq!(next.piece_at(Square::from_algebraic("h1").unwrap()), None);
    }

    #[test]
    fn en_passant_capture_removes_the_captured_pawn_not_on_the_destination() {
        // Blancs viennent de jouer e4-e5, noirs jouent d7-d5, blancs prennent en passant.
        let board = Board::from_fen("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1").unwrap();
        let mv = Move::new(
            Square::from_algebraic("e5").unwrap(),
            Square::from_algebraic("d6").unwrap(),
            MoveFlag::EnPassantCapture,
        );
        let next = board.make_move(mv);
        assert_eq!(next.piece_at(Square::from_algebraic("d5").unwrap()), None);
        assert_eq!(
            next.piece_at(Square::from_algebraic("d6").unwrap()),
            Some(Piece::new(Color::White, PieceType::Pawn))
        );
    }

    #[test]
    fn halfmove_clock_resets_on_capture_or_pawn_move_and_increments_otherwise() {
        let board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 5 10").unwrap();
        let quiet_king_move = Move::new(
            Square::from_algebraic("e1").unwrap(),
            Square::from_algebraic("d1").unwrap(),
            MoveFlag::Quiet,
        );
        let after_quiet = board.make_move(quiet_king_move);
        assert_eq!(after_quiet.halfmove_clock, 6);

        let pawn_move = Move::new(
            Square::from_algebraic("e2").unwrap(),
            Square::from_algebraic("e3").unwrap(),
            MoveFlag::Quiet,
        );
        let after_pawn = board.make_move(pawn_move);
        assert_eq!(after_pawn.halfmove_clock, 0);
    }
}
