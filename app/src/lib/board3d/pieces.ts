// Pièces Staunton en 3D : profils tournés (comme au tour à bois) pour le pion, la tour, le
// fou, la dame et le roi ; tête de cheval sculptée (silhouette extrudée et biseautée) pour
// le cavalier. Unités : une case = 1. Les géométries sont partagées par toutes les pièces.

import * as THREE from "three";
import type { PieceKind } from "@/lib/chess";

type Pt = [number, number];

/** Arc de cercle (pour les boules et têtes arrondies), de `from` à `to` degrés. */
function arc(cy: number, r: number, from: number, to: number, steps = 10): Pt[] {
  const pts: Pt[] = [];
  for (let i = 0; i <= steps; i++) {
    const a = ((from + ((to - from) * i) / steps) * Math.PI) / 180;
    pts.push([Math.max(0, Math.cos(a) * r), cy + Math.sin(a) * r]);
  }
  return pts;
}

function lathe(points: Pt[]): THREE.LatheGeometry {
  const geo = new THREE.LatheGeometry(
    points.map(([r, y]) => new THREE.Vector2(r, y)),
    48,
  );
  geo.computeVertexNormals();
  return geo;
}

// Socle commun : large, avec une moulure (le feutre est sous la pièce).
const BASE: Pt[] = [
  [0, 0],
  [0.37, 0],
  [0.37, 0.055],
  [0.33, 0.085],
  [0.32, 0.125],
  [0.26, 0.16],
];

const PROFILES: Record<Exclude<PieceKind, "n">, Pt[]> = {
  p: [...BASE.slice(0, 5).map(([r, y]): Pt => [r * 0.9, y]), [0.21, 0.17], [0.15, 0.3], [0.12, 0.44], [0.2, 0.47], [0.2, 0.5], [0.11, 0.53], ...arc(0.69, 0.17, -62, 90)],
  r: [...BASE, [0.22, 0.3], [0.21, 0.62], [0.28, 0.66], [0.285, 0.7], [0.27, 0.72], [0.27, 0.9], [0.2, 0.9], [0.2, 0.84], [0, 0.84]],
  b: [...BASE, [0.17, 0.32], [0.12, 0.62], [0.2, 0.655], [0.2, 0.69], [0.11, 0.72], ...arc(0.92, 0.19, -60, 70, 8), [0.05, 1.12], ...arc(1.17, 0.055, -60, 90, 6)],
  q: [...BASE, [0.19, 0.36], [0.13, 0.86], [0.23, 0.9], [0.23, 0.935], [0.14, 0.97], [0.17, 1.06], [0.25, 1.2], [0.21, 1.22], [0.12, 1.2], [0, 1.24]],
  k: [...BASE, [0.19, 0.36], [0.14, 0.9], [0.24, 0.94], [0.24, 0.975], [0.15, 1.01], [0.17, 1.1], [0.23, 1.22], [0.23, 1.26], [0, 1.27]],
};

/** Silhouette de la tête du cavalier (profil, museau vers -x), en unités de case. */
const KNIGHT_HEAD: Pt[] = [
  [0.27, 0.22],
  [0.3, 0.42],
  [0.27, 0.6],
  [0.22, 0.77],
  [0.15, 0.9],
  [0.08, 0.99],
  [0.02, 1.06],
  [-0.03, 0.97],
  [-0.1, 0.95],
  [-0.2, 0.88],
  [-0.29, 0.76],
  [-0.36, 0.64],
  [-0.35, 0.56],
  [-0.24, 0.55],
  [-0.13, 0.52],
  [-0.07, 0.45],
  [-0.14, 0.36],
  [-0.21, 0.27],
  [-0.22, 0.22],
];

export interface PieceParts {
  geometries: THREE.BufferGeometry[];
}

let cache: Record<PieceKind, THREE.BufferGeometry[]> | null = null;

function build(): Record<PieceKind, THREE.BufferGeometry[]> {
  const rook: THREE.BufferGeometry[] = [lathe(PROFILES.r)];
  // Créneaux de la tour : six merlons autour du sommet.
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2;
    const m = new THREE.BoxGeometry(0.11, 0.1, 0.1);
    m.rotateY(-a);
    m.translate(Math.cos(a) * 0.235, 0.95, Math.sin(a) * 0.235);
    rook.push(m);
  }

  const queen: THREE.BufferGeometry[] = [lathe(PROFILES.q)];
  // Couronne de la dame : perles autour du sommet, et une perle au centre.
  for (let i = 0; i < 9; i++) {
    const a = (i / 9) * Math.PI * 2;
    const s = new THREE.SphereGeometry(0.045, 12, 8);
    s.translate(Math.cos(a) * 0.215, 1.235, Math.sin(a) * 0.215);
    queen.push(s);
  }
  const top = new THREE.SphereGeometry(0.075, 16, 12);
  top.translate(0, 1.31, 0);
  queen.push(top);

  const king: THREE.BufferGeometry[] = [lathe(PROFILES.k)];
  const crossV = new THREE.BoxGeometry(0.07, 0.28, 0.07);
  crossV.translate(0, 1.4, 0);
  const crossH = new THREE.BoxGeometry(0.22, 0.07, 0.07);
  crossH.translate(0, 1.43, 0);
  king.push(crossV, crossH);

  // Cavalier : socle tourné + tête extrudée, biseautée pour des arêtes douces.
  const knightBase = lathe([...BASE, [0.25, 0.24], [0, 0.24]]);
  const shape = new THREE.Shape(KNIGHT_HEAD.map(([x, y]) => new THREE.Vector2(x, y)));
  const head = new THREE.ExtrudeGeometry(shape, {
    depth: 0.26,
    bevelEnabled: true,
    bevelThickness: 0.05,
    bevelSize: 0.035,
    bevelSegments: 4,
    curveSegments: 12,
  });
  head.translate(0, 0, -0.13);
  head.rotateY(-Math.PI / 2); // museau vers -z (vers l'adversaire pour les blancs)
  head.computeVertexNormals();

  return {
    p: [lathe(PROFILES.p)],
    r: rook,
    b: [lathe(PROFILES.b)],
    q: queen,
    k: king,
    n: [knightBase, head],
  };
}

export function pieceGeometries(kind: PieceKind): THREE.BufferGeometry[] {
  cache ??= build();
  return cache[kind];
}

export function disposePieceGeometries() {
  if (!cache) return;
  Object.values(cache).forEach((list) => list.forEach((g) => g.dispose()));
  cache = null;
}
