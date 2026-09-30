// Apparence de Chess Academy : trois réglages indépendants, combinables librement.
//  - le thème de l'interface (couleurs, polices, décor) ;
//  - le modèle d'échiquier ;
//  - le jeu de pièces.
// Chaque thème propose un plateau et des pièces par défaut.

export type ThemeId = "classic" | "persona5" | "persona3" | "sumi";
export type BoardId = "wood" | "marble" | "crystal" | "persona5" | "persona3" | "sumi";
export type PieceSetId = "staunton" | "crystal" | "holo" | "persona" | "ink";

export interface ThemeInfo {
  id: ThemeId;
  name: string;
  tagline: string;
  board: BoardId;
  pieces: PieceSetId;
  /** Trois couleurs pour l'aperçu du sélecteur. */
  swatch: [string, string, string];
}

export const THEMES: ThemeInfo[] = [
  {
    id: "classic",
    name: "Classique",
    tagline: "Ivoire, beige et ébène : l'élégance d'un vrai échiquier de club.",
    board: "wood",
    pieces: "staunton",
    swatch: ["#f4efe6", "#d9c3a0", "#1b1814"],
  },
  {
    id: "persona5",
    name: "Persona 5",
    tagline: "Rouge sang, noir et blanc, angles vifs : le style des Voleurs Fantômes.",
    board: "persona5",
    pieces: "persona",
    swatch: ["#d4001f", "#111111", "#ffffff"],
  },
  {
    id: "persona3",
    name: "Persona 3",
    tagline: "Bleu nuit et cyan électrique, comme l'Heure Sombre.",
    board: "persona3",
    pieces: "crystal",
    swatch: ["#061640", "#1f6bff", "#3fe0ff"],
  },
  {
    id: "sumi",
    name: "Encre de Chine",
    tagline: "Papier washi, lavis d'encre et sceau vermillon, comme une peinture.",
    board: "sumi",
    pieces: "ink",
    swatch: ["#efe6d2", "#1b1b1b", "#b8321f"],
  },
];

export const BOARDS: { id: BoardId; name: string; light: string; dark: string }[] = [
  { id: "wood", name: "Bois & ébène", light: "#e8d4b0", dark: "#2a211b" },
  { id: "marble", name: "Marbre", light: "#f1efea", dark: "#1c1c1f" },
  { id: "crystal", name: "Cristal", light: "#e9f2f7", dark: "#0b0d14" },
  { id: "persona5", name: "Persona 5", light: "#f4f4f4", dark: "#c8001e" },
  { id: "persona3", name: "Persona 3", light: "#cfe3ff", dark: "#1a4bbd" },
  { id: "sumi", name: "Lavis d'encre", light: "#efe6d2", dark: "#3a3733" },
];

export const PIECE_SETS: { id: PieceSetId; name: string; description: string }[] = [
  { id: "staunton", name: "Staunton", description: "Buis et ébène tournés, le modèle de tournoi." },
  { id: "crystal", name: "Cristal", description: "Verre taillé transparent, reflets et éclats." },
  { id: "holo", name: "Holographique", description: "Verre irisé aux reflets néon." },
  { id: "persona", name: "Persona", description: "Aplats nets, contour épais et ombre rouge." },
  { id: "ink", name: "Encre", description: "Silhouettes peintes au pinceau." },
];

export function themeInfo(id: ThemeId): ThemeInfo {
  return THEMES.find((t) => t.id === id) ?? THEMES[0];
}
