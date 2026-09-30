//! Pont entre l'interface et le moteur d'échecs : chaque commande reçoit une position FEN
//! (l'interface garde l'historique) et renvoie de quoi l'afficher.
//!
//! Les fonctions `api_*` sont indépendantes de Tauri : elles servent aux commandes de
//! l'application, à la passerelle de développement (`examples/dev_bridge.rs`) et aux tests.

use chess_engine::coach::{engine_move, review_move};
use chess_engine::movegen::legal_moves;
use chess_engine::san::{parse_uci, to_san, Language};
use chess_engine::{Board, Color, PieceType};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct LegalMove {
    pub uci: String,
    pub from: String,
    pub to: String,
    pub san: String,
    pub promotion: Option<char>,
}

#[derive(Serialize)]
pub struct PositionInfo {
    pub fen: String,
    pub turn: char,
    pub legal: Vec<LegalMove>,
    pub check: bool,
    pub checkmate: bool,
    pub stalemate: bool,
    pub insufficient_material: bool,
    /// Case du roi au trait, pour l'éclairer quand il est en échec.
    pub king_square: Option<String>,
}

#[derive(Serialize)]
pub struct Played {
    pub san: String,
    pub uci: String,
    pub captured: Option<char>,
    pub position: PositionInfo,
}

#[derive(Serialize)]
pub struct EngineMove {
    pub uci: String,
    pub san: String,
}

#[derive(Serialize)]
pub struct Review {
    pub quality: String,
    pub label: String,
    pub loss: i32,
    pub best_uci: Option<String>,
    pub best_san: Option<String>,
    pub eval_white: i32,
    pub messages: Vec<String>,
}

fn board(fen: &str) -> Result<Board, String> {
    Board::from_fen(fen).map_err(|e| format!("Position invalide : {e}"))
}

/// Matériel insuffisant pour mater : roi seul, ou roi + une pièce mineure contre roi seul.
fn insufficient_material(b: &Board) -> bool {
    let mut minors = 0;
    for color in [Color::White, Color::Black] {
        for (_, p) in b.pieces_of(color) {
            match p.kind {
                PieceType::King => {}
                PieceType::Knight | PieceType::Bishop => minors += 1,
                _ => return false,
            }
        }
    }
    minors <= 1
}

pub fn describe(b: &Board) -> PositionInfo {
    let moves = legal_moves(b);
    let check = b.is_in_check(b.side_to_move);
    PositionInfo {
        fen: b.to_fen(),
        turn: if b.side_to_move == Color::White { 'w' } else { 'b' },
        legal: moves
            .iter()
            .map(|&m| LegalMove {
                uci: m.to_uci(),
                from: m.from.to_algebraic(),
                to: m.to.to_algebraic(),
                san: to_san(b, m, Language::French),
                promotion: m.flag.promotion().map(|p| p.to_fen_char()),
            })
            .collect(),
        check,
        checkmate: check && moves.is_empty(),
        stalemate: !check && moves.is_empty(),
        insufficient_material: insufficient_material(b),
        king_square: b.king_square(b.side_to_move).map(|s| s.to_algebraic()),
    }
}

pub fn api_position(fen: &str) -> Result<PositionInfo, String> {
    Ok(describe(&board(fen)?))
}

pub fn api_play(fen: &str, uci: &str) -> Result<Played, String> {
    let b = board(fen)?;
    let mv = parse_uci(&b, uci).ok_or("Ce coup n'est pas autorisé ici.")?;
    let captured = if mv.flag.is_capture() {
        Some(if mv.flag == chess_engine::MoveFlag::EnPassantCapture {
            'p'
        } else {
            b.piece_at(mv.to).map(|p| p.kind.to_fen_char()).unwrap_or('p')
        })
    } else {
        None
    };
    let san = to_san(&b, mv, Language::French);
    let next = b.make_move(mv);
    Ok(Played { san, uci: uci.to_string(), captured, position: describe(&next) })
}

pub fn api_engine_reply(fen: &str, level: u8, seed: u64) -> Result<Option<EngineMove>, String> {
    let b = board(fen)?;
    Ok(engine_move(&b, level, seed).map(|m| EngineMove { uci: m.to_uci(), san: to_san(&b, m, Language::French) }))
}

pub fn api_hint(fen: &str) -> Result<Option<EngineMove>, String> {
    api_engine_reply(fen, 5, 1)
}

pub fn api_review(fen: &str, uci: &str) -> Result<Review, String> {
    let b = board(fen)?;
    let mv = parse_uci(&b, uci).ok_or("Coup inconnu.")?;
    let r = review_move(&b, mv, 3);
    Ok(Review {
        quality: r.quality.id().into(),
        label: r.quality.label().into(),
        loss: r.loss,
        best_uci: r.best_move.map(|m| m.to_uci()),
        best_san: r.best_san,
        eval_white: r.eval_after_white,
        messages: r.messages,
    })
}

fn json<T: Serialize>(v: Result<T, String>) -> Result<Value, String> {
    v.map(|x| serde_json::to_value(x).unwrap_or(Value::Null))
}

/// Aiguillage générique « nom de commande + arguments JSON » (passerelle de développement).
pub fn dispatch(cmd: &str, args: &Value) -> Result<Value, String> {
    let s = |k: &str| args.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let n = |k: &str| args.get(k).and_then(Value::as_u64).unwrap_or(0);
    match cmd {
        "position" => json(api_position(&s("fen"))),
        "play" => json(api_play(&s("fen"), &s("uci"))),
        "engine_reply" => json(api_engine_reply(&s("fen"), n("level") as u8, n("seed"))),
        "hint" => json(api_hint(&s("fen"))),
        "review" => json(api_review(&s("fen"), &s("uci"))),
        _ => Err(format!("Commande inconnue : {cmd}")),
    }
}

#[tauri::command]
fn position(fen: String) -> Result<PositionInfo, String> {
    api_position(&fen)
}

#[tauri::command]
fn play(fen: String, uci: String) -> Result<Played, String> {
    api_play(&fen, &uci)
}

#[tauri::command(async)]
fn engine_reply(fen: String, level: u8, seed: u64) -> Result<Option<EngineMove>, String> {
    api_engine_reply(&fen, level, seed)
}

#[tauri::command(async)]
fn hint(fen: String) -> Result<Option<EngineMove>, String> {
    api_hint(&fen)
}

#[tauri::command(async)]
fn review(fen: String, uci: String) -> Result<Review, String> {
    api_review(&fen, &uci)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![position, play, engine_reply, hint, review])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer Chess Academy");
}
