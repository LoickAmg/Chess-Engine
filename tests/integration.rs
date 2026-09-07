//! Tests d'intégration en boîte noire : uniquement l'API publique du
//! crate, comme si on l'utilisait depuis un autre programme. Complètent
//! les tests unitaires (qui vérifient chaque module isolément) par des
//! scénarios de bout en bout — jouer une vraie partie, piloter le moteur
//! via le protocole UCI texte.

use chess_engine::board::Board;
use chess_engine::movegen::legal_moves;
use chess_engine::moves::MoveFlag;
use chess_engine::perft::perft;
use chess_engine::search::{self, SearchLimits};
use chess_engine::uci::UciEngine;
use chess_engine::Color;

fn play_uci_moves(board: &Board, moves: &[&str]) -> Board {
    let mut current = board.clone();
    for &uci in moves {
        let mv = legal_moves(&current)
            .into_iter()
            .find(|m| m.to_uci() == uci)
            .unwrap_or_else(|| panic!("coup illégal ou introuvable dans cette position : {uci}"));
        current = current.make_move(mv);
    }
    current
}

#[test]
fn scholars_mate_is_detected_as_checkmate() {
    // 1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6?? 4. Qxf7# — le mat du berger,
    // l'un des pièges d'ouverture les plus connus.
    let board = Board::starting_position();
    let final_position = play_uci_moves(
        &board,
        &["e2e4", "e7e5", "f1c4", "b8c6", "d1h5", "g8f6", "h5f7"],
    );
    assert!(final_position.is_in_check(Color::Black));
    assert!(
        legal_moves(&final_position).is_empty(),
        "le mat du berger devrait laisser le camp noir sans aucun coup légal"
    );
}

#[test]
fn perft_via_the_public_api_matches_the_known_startpos_value_at_depth_four() {
    let board = Board::starting_position();
    assert_eq!(perft(&board, 4), 197_281);
}

#[test]
fn engine_plays_a_short_self_play_game_without_ever_producing_an_illegal_position() {
    let mut board = Board::starting_position();
    let mut plies_played = 0;

    for _ in 0..30 {
        let moves = legal_moves(&board);
        if moves.is_empty() {
            break; // Mat ou pat : fin de partie normale.
        }
        let info = search::search(&board, SearchLimits::depth(2));
        let mv = info
            .best_move
            .expect("une recherche à profondeur 2 doit renvoyer un coup dès qu'il en existe un");
        assert!(
            moves.contains(&mv),
            "le coup renvoyé par la recherche doit toujours être légal"
        );
        board = board.make_move(mv);
        plies_played += 1;

        // Invariant qui doit rester vrai après CHAQUE coup légal : le camp
        // qui vient de jouer ne doit jamais avoir laissé son propre roi en
        // échec (sans quoi la recherche n'a pas filtré correctement).
        let mover = if board.side_to_move == Color::White {
            Color::Black
        } else {
            Color::White
        };
        assert!(!board.is_in_check(mover));
    }

    assert!(
        plies_played >= 10,
        "la partie devrait durer au moins quelques coups avant un éventuel mat rapide"
    );
}

#[test]
fn uci_protocol_can_drive_an_entire_short_game() {
    let mut engine = UciEngine::new();
    engine.handle_line("uci");
    engine.handle_line("isready");
    engine.handle_line("ucinewgame");
    engine.handle_line("position startpos");

    let mut moves_played: Vec<String> = Vec::new();
    for _ in 0..6 {
        let response = engine.handle_line("go depth 2");
        let bestmove_line = response
            .iter()
            .find(|l| l.starts_with("bestmove"))
            .expect("chaque 'go' doit renvoyer une ligne bestmove");
        let mv = bestmove_line.strip_prefix("bestmove ").unwrap().to_string();
        if mv == "0000" {
            break; // Plus aucun coup légal (mat/pat).
        }
        moves_played.push(mv);
        let position_cmd = format!("position startpos moves {}", moves_played.join(" "));
        engine.handle_line(&position_cmd);
    }

    assert!(
        moves_played.len() >= 4,
        "le protocole UCI devrait pouvoir enchaîner plusieurs coups sans erreur"
    );
}

#[test]
fn a_pawn_promotion_played_through_make_move_produces_the_chosen_piece() {
    let board = Board::from_fen("8/4P3/8/8/8/8/8/k6K w - - 0 1").unwrap();
    let promotion_move = legal_moves(&board)
        .into_iter()
        .find(|mv| mv.flag == MoveFlag::Promotion(chess_engine::PieceType::Queen))
        .expect("la promotion en dame doit être un coup légal disponible");
    let next = board.make_move(promotion_move);
    assert_eq!(
        next.piece_at(chess_engine::Square::from_algebraic("e8").unwrap()),
        Some(chess_engine::Piece::new(
            Color::White,
            chess_engine::PieceType::Queen
        ))
    );
}
