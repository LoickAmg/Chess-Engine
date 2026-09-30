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
import ChessPiece from "./ChessPiece.vue";

// Échiquier de cristal : cases de verre, filigrane de symboles de cartes, coups en
// losanges cyan, prises en anneaux roses. Clic-clic ou glisser-déposer, choix de la
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

const SUITS = ["♠", "♥", "♦", "♣"];

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
  const out: { sq: string; light: boolean; suit: string }[] = [];
  for (let row = 0; row < 8; row++) {
    for (let col = 0; col < 8; col++) {
      const f = flipped.value ? 7 - col : col;
      const r = flipped.value ? row : 7 - row;
      const sq = squareName(f, r);
      out.push({ sq, light: (f + r) % 2 === 1, suit: SUITS[(f * 3 + r * 5) % 4] });
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
  return `M${x1} ${y1} L${x1 + (x2 - x1) * k} ${y1 + (y2 - y1) * k}`;
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
  <div class="board-wrap" :class="{ 'with-coords': showCoords }">
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
          :data-suit="s.suit"
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
    </div>

    <template v-if="showCoords">
      <div class="files">
        <span v-for="f in flipped ? [...FILES].reverse() : FILES" :key="f">{{ f }}</span>
      </div>
      <div class="ranks">
        <span v-for="r in flipped ? [1, 2, 3, 4, 5, 6, 7, 8] : [8, 7, 6, 5, 4, 3, 2, 1]" :key="r">{{ r }}</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.board-wrap {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
}
.board-wrap.with-coords {
  padding: 0 0 26px 26px;
}
.frame {
  position: relative;
  width: 100%;
  height: 100%;
  padding: 10px;
  border-radius: 22px;
  /* Cadre dégradé rose → violet → cyan, qui flotte au-dessus du vide */
  background:
    linear-gradient(var(--bg-1), var(--bg-1)) padding-box,
    conic-gradient(from 210deg, var(--pink), var(--violet), var(--cyan), var(--gold), var(--pink)) border-box;
  border: 3px solid transparent;
  box-shadow:
    0 0 0 1px rgba(255, 255, 255, 0.06),
    0 0 40px rgba(155, 107, 255, 0.35),
    0 30px 60px rgba(5, 0, 25, 0.7);
  animation: hover-float 7s ease-in-out infinite;
}
@keyframes hover-float {
  50% {
    transform: translateY(-4px);
  }
}
.board {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 12px;
  overflow: hidden;
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  grid-template-rows: repeat(8, 1fr);
  touch-action: none;
  container-type: inline-size;
}
.board.interactive,
.board.picking {
  cursor: pointer;
}

/* Cases de verre */
.sq {
  position: relative;
}
.sq.light {
  background: linear-gradient(145deg, #f3ecff 0%, #d9cdfa 55%, #c9e6ff 100%);
}
.sq.dark {
  background: linear-gradient(145deg, #6a3fc9 0%, #4a2598 55%, #3a1f86 100%);
}
.sq::before {
  /* Reflet de verre sur le haut de chaque case */
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(160deg, rgba(255, 255, 255, 0.28), transparent 45%);
  pointer-events: none;
}
.sq.dark::after {
  /* Filigrane de symbole de carte */
  content: attr(data-suit);
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  font-size: 4.2cqw;
  color: rgba(255, 255, 255, 0.07);
  pointer-events: none;
}
.sq.light::after {
  content: "";
  position: absolute;
  inset: 0;
  box-shadow: inset 0 0 0 0.5px rgba(120, 90, 200, 0.18);
}
.sq.hover {
  box-shadow: inset 0 0 0 3px rgba(50, 224, 255, 0.9);
}
.sq-name {
  position: absolute;
  left: 6%;
  bottom: 4%;
  font-family: var(--font-display);
  font-size: 2.1cqw;
  font-weight: 600;
  z-index: 1;
}
.light .sq-name {
  color: #5a33b0;
}
.dark .sq-name {
  color: #e6dcff;
}

/* Calques (surbrillances, coups, étoiles) */
.mark,
.hint,
.star {
  position: absolute;
  width: 12.5%;
  height: 12.5%;
  pointer-events: none;
}
.mark.last {
  background: rgba(255, 209, 102, 0.42);
}
.mark.selected {
  background: rgba(50, 224, 255, 0.35);
  box-shadow: inset 0 0 0 3px rgba(50, 224, 255, 0.9);
}
.mark.check {
  background: radial-gradient(circle, rgba(255, 60, 90, 0.95) 0%, rgba(255, 60, 90, 0.5) 35%, transparent 70%);
  animation: pulse 1.1s ease-in-out infinite;
}
.mark.good {
  background: rgba(69, 227, 160, 0.5);
  box-shadow: inset 0 0 0 3px rgba(69, 227, 160, 0.95);
}
.mark.bad {
  background: rgba(255, 93, 115, 0.5);
  box-shadow: inset 0 0 0 3px rgba(255, 93, 115, 0.95);
}
.mark.info {
  background: rgba(50, 224, 255, 0.32);
}
.mark.focus {
  box-shadow: inset 0 0 0 4px var(--gold), 0 0 20px rgba(255, 209, 102, 0.8);
  animation: pulse 1.4s ease-in-out infinite;
  z-index: 3;
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
  width: 26%;
  height: 26%;
  transform: rotate(45deg);
  border-radius: 3px;
  background: linear-gradient(135deg, #b8f6ff, var(--cyan));
  box-shadow: 0 0 10px rgba(50, 224, 255, 0.9);
  animation: pop-in 0.2s var(--spring);
}
.hint.capture::after {
  content: "";
  width: 86%;
  height: 86%;
  border-radius: 50%;
  border: 0.9cqw solid rgba(255, 79, 163, 0.9);
  box-shadow: 0 0 12px rgba(255, 79, 163, 0.7);
}
.star {
  z-index: 4;
  display: grid;
  place-items: center;
}
.star svg {
  width: 62%;
  height: 62%;
  fill: var(--gold);
  filter: drop-shadow(0 0 8px rgba(255, 209, 102, 0.9));
  animation: twinkle 1.6s ease-in-out infinite;
}
@keyframes twinkle {
  50% {
    transform: scale(0.86) rotate(12deg);
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
  transition: transform 0.26s var(--ease);
  will-change: transform;
}
.piece-slot :deep(.piece) {
  width: 86%;
  height: 86%;
  transition: transform 0.2s var(--spring);
}
.piece-slot.lifted :deep(.piece) {
  transform: translateY(-6%) scale(1.06);
}
.piece-slot.dragging {
  z-index: 30;
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
  stroke-width: 0.17;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.88;
  filter: drop-shadow(0 0 0.08px rgba(0, 0, 0, 0.5));
}
.stroke-cyan {
  stroke: var(--cyan);
}
.stroke-pink {
  stroke: var(--pink);
}
.stroke-gold {
  stroke: var(--gold);
}
.stroke-green {
  stroke: var(--green);
}
.fill-cyan {
  fill: var(--cyan);
}
.fill-pink {
  fill: var(--pink);
}
.fill-gold {
  fill: var(--gold);
}
.fill-green {
  fill: var(--green);
}

/* Promotion */
.promo-scrim {
  position: absolute;
  inset: 0;
  z-index: 40;
  background: rgba(10, 2, 30, 0.5);
}
.promo {
  position: absolute;
  width: 12.5%;
  height: 50%;
  display: grid;
  grid-template-rows: repeat(4, 1fr);
  border-radius: 12px;
  overflow: hidden;
  background: var(--panel-strong);
  box-shadow: 0 0 0 2px var(--cyan), var(--shadow);
  animation: pop-in 0.2s var(--spring);
}
.promo-btn {
  padding: 8%;
  border: 0;
  background: none;
}
.promo-btn:hover {
  background: rgba(50, 224, 255, 0.25);
}

/* Coordonnées dorées */
.files,
.ranks {
  position: absolute;
  display: flex;
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 13px;
  color: var(--gold);
  text-shadow: 0 0 8px rgba(255, 209, 102, 0.5);
}
.files {
  left: 26px;
  right: 0;
  bottom: 0;
  height: 22px;
  padding: 0 13px;
}
.files span,
.ranks span {
  flex: 1;
  display: grid;
  place-items: center;
}
.ranks {
  top: 0;
  bottom: 26px;
  left: 0;
  width: 22px;
  flex-direction: column;
  padding: 13px 0;
}
</style>
