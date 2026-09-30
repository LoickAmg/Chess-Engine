import { describe, expect, it } from "vitest";
import { freeMoves, parseFen, repetitionKey, START_FEN, toFen } from "@/lib/chess";
import { CHAPTERS, LESSONS, rich } from "@/lib/content";

describe("outils d'échiquier", () => {
  it("FEN : lecture puis réécriture du placement", () => {
    const { board, turn } = parseFen(START_FEN);
    expect(turn).toBe("w");
    expect(board.get("e1")).toEqual({ color: "w", kind: "k" });
    expect(toFen(board).split(" ")[0]).toBe(START_FEN.split(" ")[0]);
  });

  it("déplacements libres : tour, cavalier au coin, pion au départ", () => {
    const rook = parseFen("8/8/8/8/3R4/8/8/8 w - - 0 1").board;
    expect(freeMoves(rook, "d4")).toHaveLength(14);
    const knight = parseFen("8/8/8/8/8/8/8/N7 w - - 0 1").board;
    expect(freeMoves(knight, "a1").sort()).toEqual(["b3", "c2"]);
    const pawn = parseFen("8/8/8/8/8/8/4P3/8 w - - 0 1").board;
    expect(freeMoves(pawn, "e2").sort()).toEqual(["e3", "e4"]);
  });

  it("clé de répétition sans les compteurs de coups", () => {
    expect(repetitionKey("8/8/8/8/8/8/8/K6k w - - 12 40")).toBe("8/8/8/8/8/8/8/K6k w - -");
  });
});

describe("contenu", () => {
  it("les étoiles de chaque exercice sont atteignables par la pièce", () => {
    for (const lesson of LESSONS) {
      for (const step of lesson.steps) {
        if (step.type !== "stars") continue;
        // Parcours en largeur : chaque étoile doit être atteignable depuis la position.
        const { board } = parseFen(step.fen);
        const [from] = [...board.keys()];
        const seen = new Set([from]);
        const queue = [from];
        while (queue.length) {
          const sq = queue.shift()!;
          const b = parseFen(step.fen).board;
          const piece = b.get(from)!;
          b.delete(from);
          b.set(sq, piece);
          for (const next of freeMoves(b, sq)) if (!seen.has(next)) (seen.add(next), queue.push(next));
        }
        for (const star of step.stars) expect(seen.has(star), `${lesson.id} : étoile ${star}`).toBe(true);
      }
    }
  });

  it("chapitres dans l'ordre du parcours", () => {
    expect(CHAPTERS[0]).toBe("Premiers pas");
    expect(CHAPTERS.at(-1)).toBe("Finales");
  });

  it("mise en forme des textes, HTML échappé", () => {
    expect(rich("**gras** et *accent* <b>")).toBe("<strong>gras</strong> et <em>accent</em> &lt;b&gt;");
  });
});
