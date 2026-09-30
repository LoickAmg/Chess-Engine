import { invoke } from "@tauri-apps/api/core";

export interface LegalMove {
  uci: string;
  from: string;
  to: string;
  san: string;
  promotion: string | null;
}

export interface PositionInfo {
  fen: string;
  turn: "w" | "b";
  legal: LegalMove[];
  check: boolean;
  checkmate: boolean;
  stalemate: boolean;
  insufficient_material: boolean;
  king_square: string | null;
}

export interface Played {
  san: string;
  uci: string;
  captured: string | null;
  position: PositionInfo;
}

export interface EngineMove {
  uci: string;
  san: string;
}

export type QualityId = "best" | "excellent" | "good" | "inaccuracy" | "mistake" | "blunder";

export interface Review {
  quality: QualityId;
  label: string;
  loss: number;
  best_uci: string | null;
  best_san: string | null;
  eval_white: number;
  messages: string[];
}

export const inDesktopApp = "__TAURI_INTERNALS__" in window;

// Hors de l'application (npm run dev dans un navigateur), les commandes passent par la
// passerelle de développement : `cargo run --release --example dev_bridge`.
async function call<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  if (inDesktopApp) return invoke<T>(cmd, args);
  const res = await fetch(`http://127.0.0.1:1431/${cmd}`, { method: "POST", body: JSON.stringify(args) });
  const data = await res.json();
  if (!res.ok) throw new Error(String(data));
  return data as T;
}

export const api = {
  position: (fen: string) => call<PositionInfo>("position", { fen }),
  play: (fen: string, uci: string) => call<Played>("play", { fen, uci }),
  engineReply: (fen: string, level: number) =>
    call<EngineMove | null>("engine_reply", { fen, level, seed: Math.floor(Math.random() * 2 ** 31) }),
  hint: (fen: string) => call<EngineMove | null>("hint", { fen }),
  review: (fen: string, uci: string) => call<Review>("review", { fen, uci }),
};
