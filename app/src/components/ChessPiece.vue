<script setup lang="ts">
import { computed } from "vue";
import type { Color, PieceKind } from "@/lib/chess";
import type { PieceSetId } from "@/lib/themes";
import { useProgressStore } from "@/stores/progress";

// Pièces de style Staunton dessinées à la main (grille 100 × 100, socle en bas).
// Une même silhouette, cinq matières : buis et ébène, cristal, verre irisé, Persona, encre.
const props = defineProps<{ kind: PieceKind; color: Color; set?: PieceSetId }>();
const progress = useProgressStore();
const set = computed<PieceSetId>(() => props.set ?? progress.settings.pieces);

// --- silhouettes : chaque pièce est une liste de formes (dessinées de l'arrière vers l'avant)
const PLINTH = "M21 94 C21 88.5 24.5 86 30 86 L70 86 C75.5 86 79 88.5 79 94 Z";
const RING = "M27 86.5 C27 82 31 80 36 80 L64 80 C69 80 73 82 73 86.5 Z";
const COLLAR = (y: number, w: number) =>
  `M${50 - w} ${y} C${50 - w} ${y - 4.5} ${50 + w} ${y - 4.5} ${50 + w} ${y} C${50 + w} ${y + 3.5} ${50 - w} ${y + 3.5} ${50 - w} ${y} Z`;

interface Part {
  d: string;
  /** Forme de détail (trait seul), pas de remplissage */
  line?: boolean;
}

const SHAPES: Record<PieceKind, Part[]> = {
  p: [
    { d: "M41.5 50 C41.5 61 37 71 33.5 80.5 L66.5 80.5 C63 71 58.5 61 58.5 50 Z" },
    { d: COLLAR(49, 15) },
    { d: "M50 17 A13.5 13.5 0 1 1 49.99 17 Z" },
    { d: RING },
    { d: PLINTH },
  ],
  r: [
    { d: "M37 80.5 C38 68 39.2 59 39.5 51 L60.5 51 C60.8 59 62 68 63 80.5 Z" },
    { d: COLLAR(52, 16) },
    { d: "M31.5 49 L31.5 22 L40 22 L40 29 L46 29 L46 22 L54 22 L54 29 L60 29 L60 22 L68.5 22 L68.5 49 Z" },
    { d: "M33 35 L67 35", line: true },
    { d: RING },
    { d: PLINTH },
  ],
  n: [
    {
      d: "M33.5 80.5 C33.5 72 36.5 66 42.5 60.5 C37 60.5 31 62 27 59.5 C22.5 56.5 21.5 51 24.5 47 C28.5 42 34.5 39 38.5 35 C40.5 28 40.5 21.5 45 14 C47.5 17 49 19.5 50.5 22 C57.5 18.5 66.5 20.5 71.5 27.5 C77.5 36 78.5 48.5 74.5 58.5 C71.5 66 68.5 72.5 68.5 80.5 Z",
    },
    { d: "M52.5 22.5 C61 27 66.5 35.5 67.5 47 C68.5 57 65 66 63 76", line: true },
    { d: "M29 51.5 C30.5 52.5 32 52.5 33 51.5", line: true },
    { d: RING },
    { d: PLINTH },
  ],
  b: [
    { d: "M42 57 C42 67 37 74.5 33 80.5 L67 80.5 C63 74.5 58 67 58 57 Z" },
    { d: COLLAR(57.5, 15) },
    { d: "M50 17 C61.5 25.5 66.5 37 62.5 46.5 C60.5 51 56 54 50 54 C44 54 39.5 51 37.5 46.5 C33.5 37 38.5 25.5 50 17 Z" },
    { d: "M44.5 39 L57.5 27", line: true },
    { d: "M50 7.5 A4.8 4.8 0 1 1 49.99 7.5 Z" },
    { d: RING },
    { d: PLINTH },
  ],
  q: [
    { d: "M42 59 C42 68 36.5 75 32.5 80.5 L67.5 80.5 C63.5 75 58 68 58 59 Z" },
    { d: COLLAR(59, 17) },
    { d: "M36 56 C35 47 32 38.5 29.5 28.5 L39.5 38.5 L44.5 22.5 L50 36.5 L55.5 22.5 L60.5 38.5 L70.5 28.5 C68 38.5 65 47 64 56 Z" },
    { d: "M29.5 22.5 A3.6 3.6 0 1 1 29.49 22.5 Z" },
    { d: "M44.5 16 A3.6 3.6 0 1 1 44.49 16 Z" },
    { d: "M55.5 16 A3.6 3.6 0 1 1 55.49 16 Z" },
    { d: "M70.5 22.5 A3.6 3.6 0 1 1 70.49 22.5 Z" },
    { d: "M50 9 A4.4 4.4 0 1 1 49.99 9 Z" },
    { d: RING },
    { d: PLINTH },
  ],
  k: [
    { d: "M42 59 C42 68 36.5 75 32.5 80.5 L67.5 80.5 C63.5 75 58 68 58 59 Z" },
    { d: COLLAR(59, 17) },
    { d: "M36 56 C34.5 47 32.5 39 32 31 C40 27 60 27 68 31 C67.5 39 65.5 47 64 56 Z" },
    { d: "M37.5 31 C39 23.5 61 23.5 62.5 31 Z" },
    { d: "M47 4.5 L53 4.5 L53 10.5 L59 10.5 L59 16 L53 16 L53 24.5 L47 24.5 L47 16 L41 16 L41 10.5 L47 10.5 Z" },
    { d: "M34 43 C44 40.5 56 40.5 66 43", line: true },
    { d: RING },
    { d: PLINTH },
  ],
};

const parts = computed(() => SHAPES[props.kind]);
const eye = computed(() => props.kind === "n");

// --- matières
const look = computed(() => {
  const w = props.color === "w";
  switch (set.value) {
    case "crystal":
      return {
        fill: `url(#pc-crystal-${props.color})`,
        stroke: w ? "#5d7d9c" : "rgba(196,206,255,0.9)",
        width: 1.5,
        detail: w ? "rgba(70,120,170,0.9)" : "rgba(220,228,255,0.8)",
        gloss: true,
        filter: "url(#pc-glow)",
        sparkle: true,
      };
    case "holo":
      return {
        fill: `url(#pc-holo-${props.color})`,
        stroke: w ? "#ffffff" : "#e6d9ff",
        width: 1.4,
        detail: w ? "rgba(255,255,255,0.9)" : "rgba(255,255,255,0.75)",
        gloss: true,
        filter: "url(#pc-glow)",
        sparkle: true,
      };
    case "persona":
      return {
        fill: w ? "#ffffff" : "#0b0b0b",
        stroke: w ? "#0b0b0b" : "#ffffff",
        width: 3,
        detail: w ? "#0b0b0b" : "#ffffff",
        gloss: false,
        filter: undefined,
        sparkle: false,
        shadow: "#e0001b",
      };
    case "ink":
      return {
        fill: `url(#pc-ink-${props.color})`,
        stroke: "#161412",
        width: w ? 2.4 : 1.8,
        detail: w ? "#161412" : "#e9dfcb",
        gloss: false,
        filter: "url(#pc-brush)",
        sparkle: false,
      };
    default:
      return {
        fill: `url(#pc-staunton-${props.color})`,
        stroke: w ? "#6b5132" : "#000000",
        width: 1.1,
        detail: w ? "rgba(107,81,50,0.8)" : "rgba(160,130,100,0.55)",
        gloss: true,
        filter: undefined,
        sparkle: false,
      };
  }
});
</script>

<template>
  <svg viewBox="0 0 100 100" class="piece" :class="[`set-${set}`, color, kind]" aria-hidden="true">
    <!-- Ombre portée au sol -->
    <ellipse cx="50" cy="94.5" rx="27" ry="3.4" class="ground" />
    <!-- Ombre graphique décalée (Persona) -->
    <g v-if="look.shadow" transform="translate(4 3)" :fill="look.shadow">
      <path v-for="(p, i) in parts.filter((x) => !x.line)" :key="`s${i}`" :d="p.d" />
    </g>
    <g :filter="look.filter" stroke-linejoin="round" stroke-linecap="round">
      <template v-for="(p, i) in parts" :key="i">
        <path v-if="p.line" :d="p.d" fill="none" :stroke="look.detail" :stroke-width="look.width * 1.1" />
        <template v-else>
          <path :d="p.d" :fill="look.fill" :stroke="look.stroke" :stroke-width="look.width" />
          <path v-if="look.gloss" :d="p.d" fill="url(#pc-gloss)" stroke="none" />
        </template>
      </template>
      <template v-if="eye">
        <ellipse cx="43.5" cy="32" rx="2.4" ry="2" :fill="look.detail" />
        <path d="M45 15 L48.5 22" fill="none" :stroke="look.detail" :stroke-width="look.width" />
      </template>
    </g>
    <!-- Éclats de lumière (verre) -->
    <g v-if="look.sparkle" class="sparkle">
      <path d="M40 30 l1.2 3.2 3.2 1.2 -3.2 1.2 -1.2 3.2 -1.2 -3.2 -3.2 -1.2 3.2 -1.2 z" />
      <path d="M58 66 l.8 2.2 2.2 .8 -2.2 .8 -.8 2.2 -.8 -2.2 -2.2 -.8 2.2 -.8 z" />
    </g>
  </svg>
</template>

<style scoped>
.piece {
  width: 100%;
  height: 100%;
  overflow: visible;
}
.ground {
  fill: rgba(0, 0, 0, 0.28);
  filter: blur(1.4px);
}
.set-staunton.b .ground {
  fill: rgba(0, 0, 0, 0.4);
}
.set-crystal .ground,
.set-holo .ground {
  fill: rgba(120, 200, 255, 0.25);
}
.set-persona .ground {
  display: none;
}
.sparkle {
  fill: #fff;
  opacity: 0.95;
  animation: glint 3.2s ease-in-out infinite;
}
.b .sparkle {
  opacity: 0.7;
}
@keyframes glint {
  0%,
  100% {
    opacity: 0.25;
  }
  50% {
    opacity: 1;
  }
}
</style>
