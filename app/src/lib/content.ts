// Contenu pédagogique : leçons, puzzles et lexique (JSON validés par les tests Rust
// de src-tauri/tests/content.rs, qui rejouent chaque coup avec le vrai moteur).

import glossaryData from "@/data/glossary.json";
import lessonsData from "@/data/lessons.json";
import puzzlesData from "@/data/puzzles.json";
import type { Arrow, Color, Highlight } from "./chess";

interface StepBase {
  text: string;
  fen?: string;
  highlights?: Highlight[];
  arrows?: Arrow[];
  orientation?: Color;
}
export type Step =
  | (StepBase & { type: "text"; squareNames?: boolean })
  | (StepBase & { type: "square"; targets: string[] })
  | (StepBase & { type: "stars"; fen: string; stars: string[] })
  | (StepBase & {
      type: "move";
      fen: string;
      /** Coups acceptés en UCI ; « * » = n'importe quel coup légal, « # » = n'importe quel mat. */
      accept: string[];
      success: string;
      wrong?: Record<string, string>;
      fallback?: string;
    })
  | (StepBase & { type: "quiz"; choices: string[]; answer: number; explain: string });

export interface Lesson {
  id: string;
  chapter: string;
  title: string;
  icon: string;
  summary: string;
  steps: Step[];
}

export interface Puzzle {
  id: string;
  title: string;
  theme: string;
  fen: string;
  /** Coups de la solution en alternance (élève, réponse, élève…), ou ["#"] : n'importe quel mat. */
  solution: string[];
  hint: string;
}

export interface GlossaryEntry {
  term: string;
  def: string;
}

export const LESSONS = lessonsData as Lesson[];
export const PUZZLES = puzzlesData as Puzzle[];
export const GLOSSARY = glossaryData as GlossaryEntry[];

export const CHAPTERS = [...new Set(LESSONS.map((l) => l.chapter))];

export const TIPS = [
  "Avant chaque coup, demande-toi : « Qu'est-ce que mon adversaire menace ? »",
  "Cavalier sur le bord, cavalier mort : garde tes cavaliers vers le centre.",
  "Roque tôt : un roi resté au centre est une cible.",
  "La dame se place sur sa couleur : dame blanche sur case claire.",
  "Ne sors pas ta dame trop tôt, elle se ferait chasser.",
  "Une pièce non protégée est une pièce en danger.",
  "En finale, le roi est une pièce d'attaque : active-le !",
  "Les pions passés doivent avancer.",
  "Quand tu as un gros avantage, échange les pièces pour simplifier.",
  "Quand l'adversaire n'a plus que son roi, attention au pat !",
  "Tours sur les colonnes ouvertes, cavaliers sur les cases avancées protégées.",
  "Un échec n'est pas toujours un bon coup : cherche celui qui améliore ta position.",
];

/** Mise en forme légère des textes : **gras**, *accent*, `code` (le HTML est échappé). */
export function rich(text: string): string {
  const escaped = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  return escaped
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`(.+?)`/g, "<code>$1</code>");
}
