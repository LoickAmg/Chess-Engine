import { defineStore } from "pinia";
import { api, type PositionInfo, type Review } from "@/lib/api";
import { halfmoveClock, repetitionKey, START_FEN, type Color } from "@/lib/chess";
import { sfx } from "@/lib/sound";
import { useProgressStore } from "./progress";

// Partie contre Lumi : l'historique est une suite de positions FEN ; le moteur (Rust)
// fournit les coups légaux, joue ses réponses et analyse chaque coup de l'élève.

export interface Ply {
  uci: string;
  san: string;
  by: "you" | "lumi";
  captured: string | null;
  /** Position après le coup. */
  fen: string;
  review?: Review;
}

export type Outcome =
  | { result: "win" | "loss" | "draw"; reason: string }
  | null;

export const LEVELS = [
  { level: 1, name: "Pion curieux", desc: "Je joue vite et je fais des erreurs. Parfait pour débuter." },
  { level: 2, name: "Cavalier malin", desc: "Je vois les prises simples, mais je laisse des occasions." },
  { level: 3, name: "Fou rusé", desc: "Je calcule un peu : attention aux pièces en prise !" },
  { level: 4, name: "Tour solide", desc: "Je joue sérieusement. Il faudra de bonnes idées." },
  { level: 5, name: "Dame redoutable", desc: "Tout ce que je sais faire. Bonne chance !" },
] as const;

export const useGameStore = defineStore("game", {
  state: () => ({
    started: false,
    level: 2,
    you: "w" as Color,
    coach: true,
    plies: [] as Ply[],
    position: null as PositionInfo | null,
    thinking: false,
    reviewing: false,
    outcome: null as Outcome,
    hint: null as { uci: string; san: string } | null,
    evalWhite: 0,
    error: null as string | null,
    token: 0,
  }),
  getters: {
    fen: (s) => s.position?.fen ?? START_FEN,
    yourTurn: (s) => !!s.position && s.position.turn === s.you && !s.outcome,
    lastMove: (s): [string, string] | null => {
      const last = s.plies[s.plies.length - 1];
      return last ? [last.uci.slice(0, 2), last.uci.slice(2, 4)] : null;
    },
    lastReview: (s) => [...s.plies].reverse().find((p) => p.by === "you" && p.review)?.review ?? null,
    lastYourPly: (s) => [...s.plies].reverse().find((p) => p.by === "you") ?? null,
    qualityCounts: (s) => {
      const counts: Record<string, number> = {};
      for (const p of s.plies) if (p.review) counts[p.review.quality] = (counts[p.review.quality] ?? 0) + 1;
      return counts;
    },
  },
  actions: {
    async start(level: number, color: Color | "random", coach: boolean) {
      this.token++;
      this.level = level;
      this.you = color === "random" ? (Math.random() < 0.5 ? "w" : "b") : color;
      this.coach = coach;
      this.plies = [];
      this.outcome = null;
      this.hint = null;
      this.evalWhite = 0;
      this.error = null;
      this.position = await api.position(START_FEN);
      this.started = true;
      useProgressStore().setSetting("level", level);
      if (this.you === "b") await this.lumiMove();
    },

    async play(uci: string) {
      if (!this.position || !this.yourTurn || this.thinking) return;
      const before = this.position.fen;
      const token = this.token;
      this.hint = null;
      try {
        const played = await api.play(before, uci);
        const ply: Ply = { uci, san: played.san, by: "you", captured: played.captured, fen: played.position.fen };
        this.plies.push(ply);
        const idx = this.plies.length - 1;
        this.position = played.position;
        this.sound(played.captured, played.position.check);
        if (this.coach) {
          this.reviewing = true;
          api
            .review(before, uci)
            .then((r) => {
              if (token !== this.token) return;
              const target = this.plies[idx];
              if (target?.uci === uci) target.review = r;
              this.evalWhite = r.eval_white;
            })
            .finally(() => (this.reviewing = false));
        }
        if (this.checkEnd()) return;
        await this.lumiMove();
      } catch (e) {
        this.error = String(e);
      }
    },

    async lumiMove() {
      if (!this.position || this.outcome) return;
      const token = this.token;
      this.thinking = true;
      const started = performance.now();
      try {
        const reply = await api.engineReply(this.position.fen, this.level);
        // Un petit temps de réflexion visible, même quand le moteur répond instantanément.
        const wait = 450 - (performance.now() - started);
        if (wait > 0) await new Promise((r) => setTimeout(r, wait));
        if (token !== this.token || !reply || !this.position) return;
        const played = await api.play(this.position.fen, reply.uci);
        if (token !== this.token) return;
        this.plies.push({ uci: reply.uci, san: played.san, by: "lumi", captured: played.captured, fen: played.position.fen });
        this.position = played.position;
        this.sound(played.captured, played.position.check);
        this.checkEnd();
      } catch (e) {
        this.error = String(e);
      } finally {
        if (token === this.token) this.thinking = false;
      }
    },

    sound(captured: string | null, check: boolean) {
      if (check) sfx.check();
      else if (captured) sfx.capture();
      else sfx.move();
    },

    checkEnd(): boolean {
      const p = this.position;
      if (!p) return false;
      let outcome: Outcome = null;
      if (p.checkmate) {
        const winner = p.turn === "w" ? "b" : "w";
        outcome = { result: winner === this.you ? "win" : "loss", reason: "Échec et mat" };
      } else if (p.stalemate) outcome = { result: "draw", reason: "Pat" };
      else if (p.insufficient_material) outcome = { result: "draw", reason: "Matériel insuffisant" };
      else if (halfmoveClock(p.fen) >= 100) outcome = { result: "draw", reason: "Règle des 50 coups" };
      else {
        const key = repetitionKey(p.fen);
        const seen = [START_FEN, ...this.plies.map((x) => x.fen)].filter((f) => repetitionKey(f) === key).length;
        if (seen >= 3) outcome = { result: "draw", reason: "Triple répétition" };
      }
      if (outcome) this.finish(outcome);
      return !!outcome;
    },

    finish(outcome: NonNullable<Outcome>) {
      this.outcome = outcome;
      this.thinking = false;
      if (outcome.result === "win") sfx.fanfare();
      useProgressStore().recordGame({
        date: Date.now(),
        level: this.level,
        color: this.you,
        result: outcome.result,
        moves: Math.ceil(this.plies.length / 2),
      });
    },

    resign() {
      if (!this.outcome && this.started) this.finish({ result: "loss", reason: "Abandon" });
    },

    async undo() {
      if (this.thinking || !this.plies.length) return;
      this.token++;
      // Retire la réponse de Lumi puis ton dernier coup.
      if (this.plies[this.plies.length - 1].by === "lumi") this.plies.pop();
      if (this.plies.length && this.plies[this.plies.length - 1].by === "you") this.plies.pop();
      const fen = this.plies[this.plies.length - 1]?.fen ?? START_FEN;
      this.position = await api.position(fen);
      this.outcome = null;
      this.hint = null;
      if (this.position.turn !== this.you) await this.lumiMove();
    },

    async askHint() {
      if (!this.position || !this.yourTurn) return;
      const h = await api.hint(this.position.fen);
      if (h) this.hint = h;
    },

    leave() {
      this.token++;
      this.started = false;
      this.thinking = false;
    },
  },
});
