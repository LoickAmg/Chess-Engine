<script setup lang="ts">
import { computed } from "vue";
import type { Color, PieceKind } from "@/lib/chess";

// Pièces originales dessinées à la main : silhouettes façon bijoux, ornées des symboles de
// cartes (♠ roi, ♥ dame, ♦ fou, ♣ cavalier, ★ tour). Grille 100 × 100.
const props = defineProps<{ kind: PieceKind; color: Color }>();

const fill = computed(() => (props.color === "w" ? "url(#pc-white)" : "url(#pc-black)"));
const stroke = computed(() => (props.color === "w" ? "#3b1a6e" : "#ffcf5c"));
const gem = computed(() => (props.color === "w" ? "url(#pc-gem-w)" : "url(#pc-gem-b)"));

const BASE = "M22 84 Q50 78 78 84 L82 92 Q50 98 18 92 Z";
const SPADE = "M50 5 C55 12 63 16 63 22.5 C63 28 56.5 29 52.5 25.5 L54.5 32 H45.5 L47.5 25.5 C43.5 29 37 28 37 22.5 C37 16 45 12 50 5 Z";
const HEART = "M50 55 C44 50 41 47 41 43.5 C41 40.5 44 39 46.5 39 C48.3 39 49.5 40 50 41.5 C50.5 40 51.7 39 53.5 39 C56 39 59 40.5 59 43.5 C59 47 56 50 50 55 Z";
const CLUB =
  "M57 60 C55 57 55.5 54 58.5 53 C58 50 60.5 48 63 49 C65.5 48 68 50 67.5 53 C70.5 54 71 57 69 60 C67.5 62 65 61.5 64 60 L65 64 H61 L62 60 C61 61.5 58.5 62 57 60 Z";
const STAR = "M50 55 L52.6 60.6 L58.6 61.2 L54.1 65.2 L55.4 71.2 L50 68.1 L44.6 71.2 L45.9 65.2 L41.4 61.2 L47.4 60.6 Z";
</script>

<template>
  <svg viewBox="0 0 100 100" class="piece" :class="[color, kind]" aria-hidden="true">
    <g :fill="fill" :stroke="stroke" stroke-width="2.6" stroke-linejoin="round" stroke-linecap="round">
      <template v-if="kind === 'p'">
        <path d="M36 82 C38 70 42 62 44 55 L56 55 C58 62 62 70 64 82 Z" />
        <path d="M37 56 Q50 50 63 56 Q50 61 37 56 Z" />
        <circle cx="50" cy="37" r="14" />
        <path :d="BASE" />
        <circle cx="50" cy="37" r="5" :fill="gem" stroke-width="1.6" />
      </template>

      <template v-else-if="kind === 'r'">
        <path d="M34 82 L38 44 H62 L66 82 Z" />
        <path d="M27 45 L29 20 H39 V29 H45 V20 H55 V29 H61 V20 H71 L73 45 Z" />
        <path :d="BASE" />
        <path :d="STAR" :fill="gem" stroke-width="1.6" />
      </template>

      <template v-else-if="kind === 'b'">
        <path d="M38 82 C40 70 44 64 45 60 H55 C56 64 60 70 62 82 Z" />
        <path d="M50 13 C64 25 67 44 57 57 H43 C33 44 36 25 50 13 Z" />
        <path d="M45 38 L57 27" fill="none" />
        <ellipse cx="50" cy="59" rx="13" ry="4.2" />
        <circle cx="50" cy="10" r="4.2" />
        <path :d="BASE" />
        <path d="M50 64 L55 70.5 L50 77 L45 70.5 Z" :fill="gem" stroke-width="1.6" />
      </template>

      <template v-else-if="kind === 'n'">
        <path
          d="M36 84 L40 64 C36 62 32 60 29 58 L21 56 C17 55 16 50 19 47 L34 30 L39 15 L46 22 C63 18 77 31 75 51 C74 63 69 73 67 84 Z"
        />
        <path d="M52 22 C61 29 65 39 65 52" fill="none" />
        <circle cx="41" cy="34" r="3.2" :fill="stroke" stroke="none" />
        <circle cx="23.5" cy="50.5" r="1.6" :fill="stroke" stroke="none" />
        <path :d="BASE" />
        <path :d="CLUB" :fill="gem" stroke-width="1.4" />
      </template>

      <template v-else-if="kind === 'q'">
        <path d="M36 84 C38 74 40 67 40 62 H60 C60 67 62 74 64 84 Z" />
        <path d="M25 30 L34 58 H66 L75 30 L62 45 L56 21 L50 42 L44 21 L38 45 Z" />
        <ellipse cx="50" cy="61" rx="17" ry="4.2" />
        <circle cx="25" cy="27" r="4.5" />
        <circle cx="44" cy="18" r="4.5" />
        <circle cx="56" cy="18" r="4.5" />
        <circle cx="75" cy="27" r="4.5" />
        <path :d="BASE" />
        <path :d="HEART" :fill="gem" stroke-width="1.5" />
      </template>

      <template v-else>
        <path d="M36 84 C38 74 40 67 40 62 H60 C60 67 62 74 64 84 Z" />
        <path d="M31 36 C40 31 60 31 69 36 L64 58 H36 Z" />
        <path d="M37 45 C45 42 55 42 63 45" fill="none" />
        <ellipse cx="50" cy="61" rx="17" ry="4.2" />
        <path :d="SPADE" :fill="gem" stroke-width="1.6" />
        <path :d="BASE" />
      </template>
    </g>
    <!-- Reflet de verre, pour l'aspect « bijou » -->
    <path
      v-if="kind !== 'n'"
      d="M41 30 Q43 24 49 23"
      fill="none"
      stroke="#fff"
      stroke-width="2.4"
      stroke-linecap="round"
      :opacity="color === 'w' ? 0.9 : 0.35"
    />
  </svg>
</template>

<style scoped>
.piece {
  width: 100%;
  height: 100%;
  overflow: visible;
  filter: drop-shadow(0 5px 3px rgba(20, 0, 40, 0.45));
}
.piece.b {
  filter: drop-shadow(0 5px 3px rgba(0, 0, 0, 0.55)) drop-shadow(0 0 6px rgba(160, 90, 255, 0.35));
}
</style>
