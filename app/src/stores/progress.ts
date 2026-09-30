import { defineStore } from "pinia";
import type { BoardId, PieceSetId, ThemeId } from "@/lib/themes";

// Progression de l'élève, gardée dans le navigateur intégré (localStorage) : leçons
// terminées, puzzles résolus, records d'entraînement, parties jouées, réglages.

const KEY = "chess-academie:v1";

export interface GameRecord {
  date: number;
  level: number;
  color: "w" | "b";
  result: "win" | "loss" | "draw";
  moves: number;
}

interface Saved {
  lessons: Record<string, { done: boolean; step: number }>;
  puzzles: Record<string, { solved: boolean; tries: number }>;
  coordsBest: Record<string, number>;
  games: GameRecord[];
  settings: {
    sound: boolean;
    coords: boolean;
    coachTalk: boolean;
    level: number;
    theme: ThemeId;
    board: BoardId;
    pieces: PieceSetId;
  };
}

function load(): Saved {
  const empty: Saved = {
    lessons: {},
    puzzles: {},
    coordsBest: {},
    games: [],
    settings: { sound: true, coords: true, coachTalk: true, level: 2, theme: "classic", board: "wood", pieces: "staunton" },
  };
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return empty;
    const data = JSON.parse(raw) as Partial<Saved>;
    return { ...empty, ...data, settings: { ...empty.settings, ...(data.settings ?? {}) } };
  } catch {
    return empty;
  }
}

export const useProgressStore = defineStore("progress", {
  state: (): Saved => load(),
  getters: {
    lessonDone: (s) => (id: string) => !!s.lessons[id]?.done,
    lessonStep: (s) => (id: string) => s.lessons[id]?.step ?? 0,
    puzzleSolved: (s) => (id: string) => !!s.puzzles[id]?.solved,
    wins: (s) => s.games.filter((g) => g.result === "win").length,
  },
  actions: {
    save() {
      try {
        localStorage.setItem(KEY, JSON.stringify(this.$state));
      } catch {
        /* stockage indisponible : la progression reste en mémoire */
      }
    },
    setLessonStep(id: string, step: number) {
      const prev = this.lessons[id];
      this.lessons[id] = { done: prev?.done ?? false, step: Math.max(step, prev?.step ?? 0) };
      this.save();
    },
    completeLesson(id: string) {
      this.lessons[id] = { done: true, step: 0 };
      this.save();
    },
    recordPuzzle(id: string, solved: boolean) {
      const prev = this.puzzles[id] ?? { solved: false, tries: 0 };
      this.puzzles[id] = { solved: prev.solved || solved, tries: prev.tries + 1 };
      this.save();
    },
    recordCoords(mode: string, score: number): boolean {
      const best = this.coordsBest[mode] ?? 0;
      if (score > best) {
        this.coordsBest[mode] = score;
        this.save();
        return true;
      }
      return false;
    },
    recordGame(game: GameRecord) {
      this.games = [game, ...this.games].slice(0, 200);
      this.save();
    },
    setSetting<K extends keyof Saved["settings"]>(key: K, value: Saved["settings"][K]) {
      this.settings[key] = value;
      this.save();
    },
  },
});
