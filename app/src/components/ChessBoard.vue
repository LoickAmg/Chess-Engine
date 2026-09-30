<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type { LegalMove } from "@/lib/api";
import {
  FILES,
  fileOf,
  parseFen,
  rankOf,
  squareName,
  type Arrow,
  type Color,
  type Highlight,
  type Piece,
  type PieceKind,
} from "@/lib/chess";
import type { BoardId } from "@/lib/themes";
import { useProgressStore } from "@/stores/progress";
import ChessPiece from "./ChessPiece.vue";

// Échiquier : six modèles de plateau (bois, marbre, cristal, Persona 5, Persona 3, lavis
// d'encre), coordonnées gravées dans le cadre. Clic-clic ou glisser-déposer, choix de la
// pièce de promotion, flèches, étoiles à collecter et pièces qui glissent d'une case à
// l'autre (chaque pièce garde son identité d'une position à la suivante).


const props = withDefaults(
  defineProps<{
    fen: string;
    orientation?: Color;
    interactive?: boolean;
    /** Coups légaux fournis par le moteur (mode partie / exercice). */
    legal?: LegalMove[];
    /** Déplacements libres calculés par l'appelant (exercices « étoiles »). */
    mover?: ((from: string) => string[]) | null;
    lastMove?: [string, string] | null;
    check?: string | null;
    highlights?: Highlight[];
    arrows?: Arrow[];
    stars?: string[];
    showCoords?: boolean;
    squareNames?: boolean;
    /** Mode « clique une case » : aucun déplacement, chaque clic est émis. */
    pickSquares?: boolean;
    /** Modèle de plateau (par défaut : celui des réglages). */
    boardStyle?: BoardId;
  }>(),
  {
    orientation: "w",
    interactive: false,
    legal: () => [],
    mover: null,
    lastMove: null,
    check: null,
    highlights: () => [],
    arrows: () => [],
    stars: () => [],
    showCoords: true,
    squareNames: false,
    pickSquares: false,
  },
);

const emit = defineEmits<{
  (e: "move", uci: string): void;
  (e: "square", sq: string): void;
}>();

const progress = useProgressStore();
const style = computed<BoardId>(() => props.boardStyle ?? progress.settings.board);

// ------------------------------------------------------------------ pièces suivies

interface Tracked extends Piece {
  id: number;
  sq: string;
}
let nextId = 1;
const pieces = ref<Tracked[]>([]);

function distance(a: string, b: string) {
  return Math.abs(fileOf(a) - fileOf(b)) + Math.abs(rankOf(a) - rankOf(b));
}

function sync(fen: string) {
  const board = parseFen(fen).board;
  const old = pieces.value;
  const next: Tracked[] = [];
  const leftovers: Tracked[] = [];
  const placed = new Set<string>();
  for (const p of old) {
    const now = board.get(p.sq);
    if (now && now.color === p.color && now.kind === p.kind) {
      next.push(p);
      placed.add(p.sq);
    } else leftovers.push(p);
  }
  for (const [sq, piece] of board) {
    if (placed.has(sq)) continue;
    let best = -1;
    let bestD = Infinity;
    leftovers.forEach((p, i) => {
      if (p.color !== piece.color || p.kind !== piece.kind) return;
      const d = distance(p.sq, sq);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    });
    if (best >= 0) {
      const [p] = leftovers.splice(best, 1);
      next.push({ ...p, sq });
    } else next.push({ id: nextId++, sq, ...piece });
  }
  pieces.value = next;
}
watch(() => props.fen, sync, { immediate: true });

const byId = computed(() => new Map(pieces.value.map((p) => [p.sq, p])));
const turn = computed(() => parseFen(props.fen).turn);

// ------------------------------------------------------------------ géométrie

const root = ref<HTMLElement | null>(null);
const flipped = computed(() => props.orientation === "b");

function coords(sq: string) {
  const f = fileOf(sq);
  const r = rankOf(sq);
  return flipped.value ? { x: 7 - f, y: r } : { x: f, y: 7 - r };
}

const squares = computed(() => {
  const out: { sq: string; light: boolean; grain: number }[] = [];
  for (let row = 0; row < 8; row++) {
    for (let col = 0; col < 8; col++) {
      const f = flipped.value ? 7 - col : col;
      const r = flipped.value ? row : 7 - row;
      const sq = squareName(f, r);
      // Veinage : chaque case a un décalage de texture différent, comme de vraies pièces de bois ou de marbre.
      out.push({ sq, light: (f + r) % 2 === 1, grain: (f * 37 + r * 71) % 100 });
    }
  }
  return out;
});

function squareAt(clientX: number, clientY: number): string | null {
  const el = root.value;
  if (!el) return null;
  const rect = el.getBoundingClientRect();
  const col = Math.floor(((clientX - rect.left) / rect.width) * 8);
  const row = Math.floor(((clientY - rect.top) / rect.height) * 8);
  if (col < 0 || col > 7 || row < 0 || row > 7) return null;
  return flipped.value ? squareName(7 - col, row) : squareName(col, 7 - row);
}

// ------------------------------------------------------------------ interactions

const selected = ref<string | null>(null);
const promo = ref<{ from: string; to: string; color: Color } | null>(null);

function targetsFrom(from: string): string[] {
  if (props.mover) return props.mover(from);
  return [...new Set(props.legal.filter((m) => m.from === from).map((m) => m.to))];
}

function canPick(sq: string): boolean {
  if (!props.interactive || props.pickSquares) return false;
  const p = byId.value.get(sq);
  if (!p) return false;
  if (props.mover) return props.mover(sq).length > 0 || p.color === turn.value;
  return p.color === turn.value && props.legal.some((m) => m.from === sq);
}

const targets = computed(() => (selected.value ? targetsFrom(selected.value) : []));

function tryMove(from: string, to: string): boolean {
  if (!targetsFrom(from).includes(to)) return false;
  const promos = props.legal.filter((m) => m.from === from && m.to === to && m.promotion);
  selected.value = null;
  if (promos.length > 1) {
    promo.value = { from, to, color: byId.value.get(from)?.color ?? "w" };
    return true;
  }
  emit("move", promos[0]?.uci ?? `${from}${to}`);
  return true;
}

function choosePromotion(kind: PieceKind) {
  if (!promo.value) return;
  emit("move", `${promo.value.from}${promo.value.to}${kind}`);
  promo.value = null;
}

// Glisser-déposer
const drag = ref<{ sq: string; x: number; y: number; moved: boolean } | null>(null);
let start = { x: 0, y: 0 };

function onPointerDown(e: PointerEvent) {
  if (e.button !== 0 || promo.value) return;
  const sq = squareAt(e.clientX, e.clientY);
  if (!sq) return;
  if (props.pickSquares) {
    emit("square", sq);
    return;
  }
  if (!props.interactive) {
    emit("square", sq);
    return;
  }
  if (selected.value && selected.value !== sq && tryMove(selected.value, sq)) return;
  if (canPick(sq)) {
    selected.value = sq;
    start = { x: e.clientX, y: e.clientY };
    const rect = root.value!.getBoundingClientRect();
    drag.value = { sq, x: e.clientX - rect.left, y: e.clientY - rect.top, moved: false };
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {
      /* pointeur synthétique (tests) : la capture est facultative */
    }
  } else {
    selected.value = null;
    emit("square", sq);
  }
}

function onPointerMove(e: PointerEvent) {
  if (!drag.value) return;
  const rect = root.value!.getBoundingClientRect();
  const moved = drag.value.moved || Math.hypot(e.clientX - start.x, e.clientY - start.y) > 5;
  drag.value = { ...drag.value, x: e.clientX - rect.left, y: e.clientY - rect.top, moved };
}

function onPointerUp(e: PointerEvent) {
  const d = drag.value;
  drag.value = null;
  if (!d || !d.moved) return; // simple clic : la pièce reste sélectionnée
  const to = squareAt(e.clientX, e.clientY);
  if (to && to !== d.sq) tryMove(d.sq, to);
  else selected.value = null;
}

watch(
  () => props.fen,
  () => {
    selected.value = null;
    promo.value = null;
  },
);

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    selected.value = null;
    promo.value = null;
  }
}
window.addEventListener("keydown", onKey);
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));

// ------------------------------------------------------------------ calques

const hoverSq = ref<string | null>(null);
function onHover(e: PointerEvent) {
  hoverSq.value = drag.value?.moved ? squareAt(e.clientX, e.clientY) : null;
  onPointerMove(e);
}

function pieceStyle(p: Tracked) {
  if (drag.value?.moved && drag.value.sq === p.sq && root.value) {
    const size = root.value.clientWidth / 8;
    return {
      transform: `translate(${drag.value.x - size / 2}px, ${drag.value.y - size * 0.6}px) scale(1.12)`,
      transition: "none",
      zIndex: 20,
    };
  }
  const { x, y } = coords(p.sq);
  return { transform: `translate(${x * 100}%, ${y * 100}%)` };
}

function cellStyle(sq: string) {
  const { x, y } = coords(sq);
  return { left: `${x * 12.5}%`, top: `${y * 12.5}%` };
}

const occupied = (sq: string) => byId.value.has(sq);

function arrowPath(a: Arrow): string {
  const p = coords(a.from);
  const q = coords(a.to);
  const x1 = p.x + 0.5;
  const y1 = p.y + 0.5;
  const x2 = q.x + 0.5;
  const y2 = q.y + 0.5;
  const dx = Math.abs(q.x - p.x);
  const dy = Math.abs(q.y - p.y);
  if ((dx === 1 && dy === 2) || (dx === 2 && dy === 1)) {
    // Coup de cavalier : flèche en L
    const cx = dx === 2 ? x2 : x1;
    const cy = dx === 2 ? y1 : y2;
    return `M${x1} ${y1} L${cx} ${cy} L${x2} ${y2}`;
  }
  const len = Math.hypot(x2 - x1, y2 - y1);
  const k = (len - 0.28) / len;
  // La flèche part du bord de la pièce, pour ne pas la masquer.
  const s = Math.min(0.34, len / 3) / len;
  return `M${x1 + (x2 - x1) * s} ${y1 + (y2 - y1) * s} L${x1 + (x2 - x1) * k} ${y1 + (y2 - y1) * k}`;
}

const promoChoices: PieceKind[] = ["q", "r", "b", "n"];
const promoStyle = computed(() => {
  if (!promo.value) return {};
  const { x, y } = coords(promo.value.to);
  return { left: `${x * 12.5}%`, top: y < 4 ? `${y * 12.5}%` : `${(y - 3) * 12.5}%` };
});

defineExpose({ clearSelection: () => (selected.value = null) });
</script>

<template>
  <div class="board-wrap" :class="[`board-${style}`, { 'with-coords': showCoords }]">
    <div class="frame">
      <div
        ref="root"
        class="board"
        :class="{ interactive, picking: pickSquares }"
        @pointerdown="onPointerDown"
        @pointermove="onHover"
        @pointerup="onPointerUp"
        @pointercancel="drag = null"
      >
        <!-- Cases -->
        <div
          v-for="s in squares"
          :key="s.sq"
          class="sq"
          :class="[s.light ? 'light' : 'dark', { hover: hoverSq === s.sq && targets.includes(s.sq) }]"
          :style="{ '--gx': `${s.grain}%`, '--gy': `${(s.grain * 7) % 100}%` }"
          :data-sq="s.sq"
        >
          <span v-if="squareNames" class="sq-name">{{ s.sq }}</span>
        </div>

        <!-- Surbrillances -->
        <template v-if="lastMove">
          <div class="mark last" :style="cellStyle(lastMove[0])" />
          <div class="mark last" :style="cellStyle(lastMove[1])" />
        </template>
        <div v-for="h in highlights" :key="`h-${h.sq}-${h.kind}`" class="mark" :class="h.kind" :style="cellStyle(h.sq)" />
        <div v-if="selected" class="mark selected" :style="cellStyle(selected)" />
        <div v-if="check" class="mark check" :style="cellStyle(check)" />

        <!-- Étoiles -->
        <TransitionGroup name="star">
          <div v-for="s in stars" :key="`star-${s}`" class="star" :style="cellStyle(s)">
            <svg viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z" /></svg>
          </div>
        </TransitionGroup>

        <!-- Pièces -->
        <div
          v-for="p in pieces"
          :key="p.id"
          class="piece-slot"
          :class="{ dragging: drag?.moved && drag.sq === p.sq, lifted: selected === p.sq }"
          :style="pieceStyle(p)"
        >
          <ChessPiece :kind="p.kind" :color="p.color" />
        </div>

        <!-- Coups possibles -->
        <div
          v-for="t in targets"
          :key="`t-${t}`"
          class="hint"
          :class="occupied(t) ? 'capture' : 'quiet'"
          :style="cellStyle(t)"
        />

        <!-- Flèches -->
        <svg class="arrows" viewBox="0 0 8 8" aria-hidden="true">
          <defs>
            <marker v-for="c in ['cyan', 'pink', 'gold', 'green']" :id="`head-${c}`" :key="c" viewBox="0 0 10 10" refX="3" refY="5" markerWidth="3.2" markerHeight="3.2" orient="auto">
              <path d="M0 0 L10 5 L0 10 z" :class="`fill-${c}`" />
            </marker>
          </defs>
          <path
            v-for="(a, i) in arrows"
            :key="`a-${i}-${a.from}${a.to}`"
            :d="arrowPath(a)"
            class="arrow"
            :class="`stroke-${a.color ?? 'cyan'}`"
            :marker-end="`url(#head-${a.color ?? 'cyan'})`"
          />
        </svg>

        <!-- Choix de promotion -->
        <div v-if="promo" class="promo-scrim" @pointerdown.stop="promo = null">
          <div class="promo" :style="promoStyle" @pointerdown.stop>
            <button v-for="k in promoChoices" :key="k" type="button" class="promo-btn" @click="choosePromotion(k)">
              <ChessPiece :kind="k" :color="promo.color" />
            </button>
          </div>
        </div>
      </div>

      <!-- Coordonnées gravées dans le cadre -->
      <template v-if="showCoords">
        <div class="files">
          <span v-for="f in flipped ? [...FILES].reverse() : FILES" :key="f">{{ f }}</span>
        </div>
        <div class="ranks">
          <span v-for="r in flipped ? [1, 2, 3, 4, 5, 6, 7, 8] : [8, 7, 6, 5, 4, 3, 2, 1]" :key="r">{{ r }}</span>
        </div>
      </template>
      <span v-if="style === 'sumi'" class="seal" aria-hidden="true">棋</span>
    </div>
  </div>
</template>

<style scoped>
/* ---------------------------------------------------------------- structure */
.board-wrap {
  --frame-pad: 10px;
  --coord: rgba(255, 255, 255, 0.7);
  --coord-font: var(--font);
  --hint: rgba(0, 0, 0, 0.28);
  --capture: rgba(0, 0, 0, 0.32);
  --last: rgba(205, 165, 60, 0.42);
  --select: rgba(80, 160, 90, 0.45);
  position: relative;
  width: 100%;
  aspect-ratio: 1;
}
.board-wrap.with-coords {
  --frame-pad: 26px;
}
.frame {
  position: relative;
  width: 100%;
  height: 100%;
  padding: var(--frame-pad);
  border-radius: 10px;
}
.board {
  position: relative;
  width: 100%;
  height: 100%;
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  grid-template-rows: repeat(8, 1fr);
  overflow: hidden;
  touch-action: none;
  container-type: inline-size;
}
.board.interactive,
.board.picking {
  cursor: pointer;
}
.sq {
  position: relative;
  background-size: 260% 260%;
  background-position: var(--gx) var(--gy);
}
.sq.hover {
  box-shadow: inset 0 0 0 3px var(--select);
}
.sq-name {
  position: absolute;
  left: 6%;
  bottom: 4%;
  font-size: 2.2cqw;
  font-weight: 700;
  z-index: 1;
  opacity: 0.75;
}
.light .sq-name {
  color: rgba(0, 0, 0, 0.6);
}
.dark .sq-name {
  color: rgba(255, 255, 255, 0.75);
}

/* Coordonnées, gravées dans le cadre */
.files,
.ranks {
  position: absolute;
  display: flex;
  font-family: var(--coord-font);
  font-size: 12px;
  font-weight: 600;
  color: var(--coord);
  pointer-events: none;
}
.files {
  left: var(--frame-pad);
  right: var(--frame-pad);
  bottom: 0;
  height: var(--frame-pad);
}
.ranks {
  top: var(--frame-pad);
  bottom: var(--frame-pad);
  left: 0;
  width: var(--frame-pad);
  flex-direction: column;
}
.files span,
.ranks span {
  flex: 1;
  display: grid;
  place-items: center;
}

/* ---------------------------------------------------------------- 1. bois et ébène */
.board-wood .frame {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(0, 0, 0, 0.25)),
    url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='400'%3E%3Cfilter id='g'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.006 0.22' numOctaves='3' seed='3'/%3E%3CfeColorMatrix values='0 0 0 0 0.12 0 0 0 0 0.06 0 0 0 0 0.02 0.9 0 0 0 -0.3'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' fill='%235a3a22'/%3E%3Crect width='100%25' height='100%25' filter='url(%23g)'/%3E%3C/svg%3E");
  box-shadow:
    inset 0 0 0 1px rgba(255, 220, 160, 0.25),
    inset 0 2px 0 rgba(255, 255, 255, 0.12),
    0 26px 50px rgba(40, 25, 10, 0.35),
    0 3px 8px rgba(40, 25, 10, 0.25);
}
.board-wood .board {
  box-shadow: 0 0 0 2px #c9a96a, 0 0 0 3px rgba(0, 0, 0, 0.4);
}
.board-wood .sq.light {
  background-color: #e6cfa6;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='g'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.01 0.3' numOctaves='3' seed='9'/%3E%3CfeColorMatrix values='0 0 0 0 0.45 0 0 0 0 0.3 0 0 0 0 0.12 0.8 0 0 0 -0.28'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23g)'/%3E%3C/svg%3E");
}
.board-wood .sq.dark {
  background-color: #2b211a;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='g'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.01 0.3' numOctaves='3' seed='5'/%3E%3CfeColorMatrix values='0 0 0 0 0.55 0 0 0 0 0.4 0 0 0 0 0.28 0.55 0 0 0 -0.2'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23g)'/%3E%3C/svg%3E");
}
.board-wood {
  --coord: #e9d3a8;
  --coord-font: "Cormorant Garamond", serif;
  --last: rgba(214, 170, 70, 0.5);
}

/* ---------------------------------------------------------------- 2. marbre */
.board-marble .frame {
  background: linear-gradient(145deg, #3a3a3e, #121214 60%, #2a2a2e);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12), 0 26px 50px rgba(0, 0, 0, 0.35);
}
.board-marble .sq.light {
  background-color: #f1efea;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='m'%3E%3CfeTurbulence type='turbulence' baseFrequency='0.011' numOctaves='4' seed='2'/%3E%3CfeColorMatrix values='0 0 0 0 0.45 0 0 0 0 0.45 0 0 0 0 0.48 -3.2 0 0 0 0.75'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23m)'/%3E%3C/svg%3E");
}
.board-marble .sq.dark {
  background-color: #1b1b1e;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='m'%3E%3CfeTurbulence type='turbulence' baseFrequency='0.011' numOctaves='4' seed='6'/%3E%3CfeColorMatrix values='0 0 0 0 0.85 0 0 0 0 0.85 0 0 0 0 0.88 -3.2 0 0 0 0.55'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23m)'/%3E%3C/svg%3E");
}
.board-marble .sq::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(150deg, rgba(255, 255, 255, 0.18), transparent 40%);
}
.board-marble {
  --coord: #d8d8dc;
  --last: rgba(200, 180, 120, 0.45);
}

/* ---------------------------------------------------------------- 3. cristal */
.board-crystal .frame {
  background: linear-gradient(145deg, #e9eef3, #9aa6b2 35%, #f7f9fb 55%, #7c8794 80%, #d6dde4);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.7), 0 30px 60px rgba(10, 20, 40, 0.4), 0 0 40px rgba(140, 200, 255, 0.25);
}
.board-crystal .board {
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
}
.board-crystal .sq.light {
  background: linear-gradient(155deg, #ffffff 0%, #e7f1f7 45%, #cddde8 100%);
}
.board-crystal .sq.dark {
  background: linear-gradient(155deg, #2a3140 0%, #0a0c12 55%, #050608 100%);
}
.board-crystal .sq::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(160deg, rgba(255, 255, 255, 0.45), transparent 38%);
  box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.35);
}
.board-crystal {
  --coord: #1b2330;
  --hint: rgba(40, 140, 220, 0.45);
  --capture: rgba(40, 140, 220, 0.55);
  --last: rgba(120, 200, 255, 0.35);
  --select: rgba(90, 180, 255, 0.5);
}

/* ---------------------------------------------------------------- 4. Persona 5 */
.board-persona5 .frame {
  background: #0b0b0b;
  border-radius: 0;
  clip-path: polygon(0 1%, 99% 0, 100% 99.2%, 1% 100%);
}
.board-persona5 .board {
  outline: 3px solid #fff;
}
.board-persona5 .sq.light {
  background-color: #f4f4f4;
  background-image: radial-gradient(circle, rgba(0, 0, 0, 0.12) 18%, transparent 20%);
  background-size: 8px 8px;
  background-position: 0 0;
}
.board-persona5 .sq.dark {
  background: linear-gradient(135deg, #e0001b, #a30013);
}
.board-persona5 {
  --coord: #ffffff;
  --coord-font: "Anton", sans-serif;
  --hint: rgba(0, 0, 0, 0.55);
  --capture: rgba(0, 0, 0, 0.7);
  --last: rgba(255, 212, 0, 0.5);
  --select: rgba(255, 212, 0, 0.6);
}

/* ---------------------------------------------------------------- 5. Persona 3 */
.board-persona3 .frame {
  background: linear-gradient(160deg, #0a2361, #030b24);
  border-radius: 2px;
  box-shadow: inset 0 0 0 1px rgba(63, 224, 255, 0.45), 0 0 40px rgba(31, 107, 255, 0.35), 0 26px 50px rgba(0, 4, 20, 0.6);
}
.board-persona3 .sq.light {
  background: linear-gradient(160deg, #e3f0ff, #b8d5ff);
}
.board-persona3 .sq.dark {
  background: linear-gradient(160deg, #2a62d6, #123a99);
}
.board-persona3 .sq::after {
  content: "";
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(115deg, transparent 0 10px, rgba(255, 255, 255, 0.06) 10px 12px);
}
.board-persona3 {
  --coord: #3fe0ff;
  --coord-font: "Barlow Condensed", sans-serif;
  --hint: rgba(3, 11, 36, 0.45);
  --capture: rgba(63, 224, 255, 0.85);
  --last: rgba(63, 224, 255, 0.35);
  --select: rgba(63, 224, 255, 0.5);
}

/* ---------------------------------------------------------------- 6. lavis d'encre */
.board-sumi .frame {
  background:
    url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='w'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.7' numOctaves='3' seed='1'/%3E%3CfeColorMatrix values='0 0 0 0 0.4 0 0 0 0 0.32 0 0 0 0 0.2 0 0 0 0.12 0'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' fill='%23f3ead8'/%3E%3Crect width='100%25' height='100%25' filter='url(%23w)'/%3E%3C/svg%3E");
  border-radius: 2px;
  box-shadow: 0 0 0 1px rgba(30, 25, 20, 0.25), 0 20px 40px rgba(40, 30, 15, 0.2);
}
.board-sumi .board {
  box-shadow: 0 0 0 3px #1b1b1b, 0 0 0 5px rgba(27, 27, 27, 0.25);
}
.board-sumi .sq.light {
  background-color: #efe6d2;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='w'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3' seed='4'/%3E%3CfeColorMatrix values='0 0 0 0 0.45 0 0 0 0 0.38 0 0 0 0 0.26 0 0 0 0.14 0'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23w)'/%3E%3C/svg%3E");
}
.board-sumi .sq.dark {
  background-color: #4a4640;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='i'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.018' numOctaves='5' seed='8'/%3E%3CfeColorMatrix values='0 0 0 0 0.04 0 0 0 0 0.04 0 0 0 0 0.04 2.2 0 0 0 -0.7'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23i)'/%3E%3C/svg%3E");
}
.board-sumi {
  --coord: #1b1b1b;
  --coord-font: "Shippori Mincho", serif;
  --hint: rgba(184, 50, 31, 0.55);
  --capture: rgba(184, 50, 31, 0.75);
  --last: rgba(184, 50, 31, 0.22);
  --select: rgba(184, 50, 31, 0.35);
}
.seal {
  position: absolute;
  right: 4px;
  bottom: 3px;
  display: grid;
  place-items: center;
  width: 20px;
  height: 20px;
  border-radius: 3px;
  background: #b8321f;
  color: #f7eee0;
  font-family: "Yuji Syuku", serif;
  font-size: 13px;
  line-height: 1;
  transform: rotate(-4deg);
  box-shadow: 0 0 0 1px rgba(120, 20, 10, 0.4);
}

/* ---------------------------------------------------------------- calques */
.mark,
.hint,
.star {
  position: absolute;
  width: 12.5%;
  height: 12.5%;
  pointer-events: none;
}
.mark.last {
  background: var(--last);
}
.mark.selected {
  background: var(--select);
}
.mark.check {
  background: radial-gradient(circle, rgba(220, 20, 40, 0.95) 0%, rgba(220, 20, 40, 0.45) 40%, transparent 72%);
}
.mark.good {
  background: rgba(70, 160, 90, 0.45);
  box-shadow: inset 0 0 0 3px rgba(60, 150, 80, 0.95);
}
.mark.bad {
  background: rgba(200, 50, 50, 0.4);
  box-shadow: inset 0 0 0 3px rgba(200, 50, 50, 0.9);
}
.mark.info {
  background: rgba(70, 140, 220, 0.32);
}
.mark.focus {
  box-shadow: inset 0 0 0 4px #c9a24a, 0 0 18px rgba(201, 162, 74, 0.7);
  z-index: 3;
  animation: pulse 1.4s ease-in-out infinite;
}
@keyframes pulse {
  50% {
    opacity: 0.55;
  }
}
.hint {
  z-index: 6;
  display: grid;
  place-items: center;
}
.hint.quiet::after {
  content: "";
  width: 30%;
  height: 30%;
  border-radius: 50%;
  background: var(--hint);
  animation: pop-in 0.18s var(--ease);
}
.hint.capture::after {
  content: "";
  width: 92%;
  height: 92%;
  border-radius: 50%;
  border: 0.85cqw solid var(--capture);
}
.star {
  z-index: 4;
  display: grid;
  place-items: center;
}
.star svg {
  width: 60%;
  height: 60%;
  fill: #e0b645;
  stroke: rgba(0, 0, 0, 0.35);
  stroke-width: 0.6;
  filter: drop-shadow(0 0 6px rgba(240, 190, 70, 0.8));
  animation: twinkle 1.6s ease-in-out infinite;
}
@keyframes twinkle {
  50% {
    transform: scale(0.88) rotate(12deg);
  }
}
.star-leave-active {
  transition: transform 0.4s var(--ease), opacity 0.4s;
}
.star-leave-to {
  transform: scale(2.2);
  opacity: 0;
}

/* Pièces */
.piece-slot {
  position: absolute;
  left: 0;
  top: 0;
  width: 12.5%;
  height: 12.5%;
  display: grid;
  place-items: center;
  z-index: 5;
  pointer-events: none;
  transition: transform 0.24s var(--ease);
  will-change: transform;
}
.piece-slot :deep(.piece) {
  width: 90%;
  height: 90%;
  transition: transform 0.2s var(--spring);
}
.piece-slot.lifted :deep(.piece) {
  transform: translateY(-5%) scale(1.05);
}
.piece-slot.dragging {
  z-index: 30;
}
.piece-slot.dragging :deep(.piece) {
  filter: drop-shadow(0 10px 8px rgba(0, 0, 0, 0.35));
}

/* Flèches */
.arrows {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  z-index: 8;
  pointer-events: none;
  overflow: visible;
}
.arrow {
  fill: none;
  stroke-width: 0.16;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.85;
}
.stroke-cyan {
  stroke: #2f8fdc;
}
.stroke-pink {
  stroke: #d23a4a;
}
.stroke-gold {
  stroke: #e0a92e;
}
.stroke-green {
  stroke: #3f9a55;
}
.fill-cyan {
  fill: #2f8fdc;
}
.fill-pink {
  fill: #d23a4a;
}
.fill-gold {
  fill: #e0a92e;
}
.fill-green {
  fill: #3f9a55;
}

/* Promotion */
.promo-scrim {
  position: absolute;
  inset: 0;
  z-index: 40;
  background: rgba(0, 0, 0, 0.4);
}
.promo {
  position: absolute;
  width: 12.5%;
  height: 50%;
  display: grid;
  grid-template-rows: repeat(4, 1fr);
  overflow: hidden;
  border-radius: var(--radius-sm);
  background: var(--surface-solid);
  box-shadow: 0 0 0 2px var(--accent), 0 14px 30px rgba(0, 0, 0, 0.35);
  animation: pop-in 0.2s var(--ease);
}
.promo-btn {
  padding: 8%;
  border: 0;
  background: none;
}
.promo-btn:hover {
  background: var(--surface-2);
}
</style>
