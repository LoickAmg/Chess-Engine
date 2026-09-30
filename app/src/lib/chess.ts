// Outils d'échiquier côté interface : lecture de FEN pour l'affichage, cases, et
// déplacements « à vide » (sans roi ni échec) pour les exercices de découverte des pièces.

export type Color = "w" | "b";
export type PieceKind = "p" | "n" | "b" | "r" | "q" | "k";
export interface Piece {
  color: Color;
  kind: PieceKind;
}
export type BoardMap = Map<string, Piece>;

export interface Highlight {
  sq: string;
  kind: "good" | "bad" | "info" | "focus";
}
export interface Arrow {
  from: string;
  to: string;
  color?: "cyan" | "pink" | "gold" | "green";
}

export const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"] as const;
export const START_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

export const PIECE_NAMES: Record<PieceKind, string> = {
  p: "pion",
  n: "cavalier",
  b: "fou",
  r: "tour",
  q: "dame",
  k: "roi",
};
export const PIECE_VALUES: Record<PieceKind, number> = { p: 1, n: 3, b: 3, r: 5, q: 9, k: 0 };

export function squareName(file: number, rank: number): string {
  return `${FILES[file]}${rank + 1}`;
}
export function fileOf(sq: string): number {
  return sq.charCodeAt(0) - 97;
}
export function rankOf(sq: string): number {
  return Number(sq[1]) - 1;
}
export function isLightSquare(sq: string): boolean {
  return (fileOf(sq) + rankOf(sq)) % 2 === 1;
}

export function parseFen(fen: string): { board: BoardMap; turn: Color } {
  const [placement, turn] = fen.split(" ");
  const board: BoardMap = new Map();
  placement.split("/").forEach((row, i) => {
    let file = 0;
    for (const ch of row) {
      if (/\d/.test(ch)) {
        file += Number(ch);
        continue;
      }
      const color: Color = ch === ch.toUpperCase() ? "w" : "b";
      board.set(squareName(file, 7 - i), { color, kind: ch.toLowerCase() as PieceKind });
      file++;
    }
  });
  return { board, turn: (turn as Color) ?? "w" };
}

/** Reconstruit la partie « placement » + trait d'une FEN (exercices sans moteur). */
export function toFen(board: BoardMap, turn: Color = "w"): string {
  const rows: string[] = [];
  for (let r = 7; r >= 0; r--) {
    let row = "";
    let empty = 0;
    for (let f = 0; f < 8; f++) {
      const p = board.get(squareName(f, r));
      if (!p) {
        empty++;
        continue;
      }
      if (empty) row += empty;
      empty = 0;
      row += p.color === "w" ? p.kind.toUpperCase() : p.kind;
    }
    if (empty) row += empty;
    rows.push(row);
  }
  return `${rows.join("/")} ${turn} - - 0 1`;
}

/** Clé de répétition : placement, trait, roques et prise en passant (sans les compteurs). */
export function repetitionKey(fen: string): string {
  return fen.split(" ").slice(0, 4).join(" ");
}

export function halfmoveClock(fen: string): number {
  return Number(fen.split(" ")[4] ?? 0);
}

/** Cases atteignables par une pièce seule (exercices « attrape les étoiles »). */
export function freeMoves(board: BoardMap, from: string): string[] {
  const piece = board.get(from);
  if (!piece) return [];
  const f0 = fileOf(from);
  const r0 = rankOf(from);
  const out: string[] = [];
  const inside = (f: number, r: number) => f >= 0 && f < 8 && r >= 0 && r < 8;
  const add = (f: number, r: number) => {
    if (!inside(f, r)) return false;
    const sq = squareName(f, r);
    const other = board.get(sq);
    if (other && other.color === piece.color) return false;
    out.push(sq);
    return !other;
  };
  const slide = (dirs: number[][]) => {
    for (const [df, dr] of dirs) {
      let f = f0 + df;
      let r = r0 + dr;
      while (add(f, r)) {
        f += df;
        r += dr;
      }
    }
  };
  const ORTHO = [[1, 0], [-1, 0], [0, 1], [0, -1]];
  const DIAG = [[1, 1], [1, -1], [-1, 1], [-1, -1]];
  switch (piece.kind) {
    case "r":
      slide(ORTHO);
      break;
    case "b":
      slide(DIAG);
      break;
    case "q":
      slide([...ORTHO, ...DIAG]);
      break;
    case "k":
      for (const [df, dr] of [...ORTHO, ...DIAG]) add(f0 + df, r0 + dr);
      break;
    case "n":
      for (const [df, dr] of [[1, 2], [2, 1], [2, -1], [1, -2], [-1, -2], [-2, -1], [-2, 1], [-1, 2]]) add(f0 + df, r0 + dr);
      break;
    case "p": {
      const dir = piece.color === "w" ? 1 : -1;
      const start = piece.color === "w" ? 1 : 6;
      if (inside(f0, r0 + dir) && !board.get(squareName(f0, r0 + dir))) {
        out.push(squareName(f0, r0 + dir));
        if (r0 === start && !board.get(squareName(f0, r0 + 2 * dir))) out.push(squareName(f0, r0 + 2 * dir));
      }
      for (const df of [-1, 1]) {
        const sq = inside(f0 + df, r0 + dir) ? squareName(f0 + df, r0 + dir) : null;
        const target = sq ? board.get(sq) : undefined;
        if (sq && target && target.color !== piece.color) out.push(sq);
      }
      break;
    }
  }
  return out;
}
