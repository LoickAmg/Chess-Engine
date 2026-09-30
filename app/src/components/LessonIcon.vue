<script setup lang="ts">
import type { PieceKind } from "@/lib/chess";
import ChessPiece from "./ChessPiece.vue";
import Icon from "./Icon.vue";

// Icône d'une leçon : une pièce de l'échiquier (p, n, b, r, q, k) ou une icône dessinée.
defineProps<{ icon: string; size?: number }>();
const PIECES = ["p", "n", "b", "r", "q", "k"];
</script>

<template>
  <span class="lesson-icon" :style="{ width: `${size ?? 44}px`, height: `${size ?? 44}px` }">
    <ChessPiece v-if="PIECES.includes(icon)" :kind="icon as PieceKind" color="w" />
    <Icon v-else :name="icon" :size="Math.round((size ?? 44) * 0.55)" />
  </span>
</template>

<style scoped>
.lesson-icon {
  flex: none;
  display: grid;
  place-items: center;
  border-radius: 30%;
  background: linear-gradient(145deg, color-mix(in srgb, var(--accent) 25%, transparent), color-mix(in srgb, var(--accent-3) 25%, transparent));
  box-shadow: inset 0 0 0 1px var(--line-strong);
  color: var(--ink);
}
.lesson-icon :deep(.piece) {
  width: 82%;
  height: 82%;
}
</style>
