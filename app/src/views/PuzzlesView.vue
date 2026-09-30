<script setup lang="ts">
import { computed } from "vue";
import { PUZZLES } from "@/lib/content";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import ChessBoard from "@/components/ChessBoard.vue";
import Icon from "@/components/Icon.vue";

const ui = useUiStore();
const progress = useProgressStore();
const solved = computed(() => PUZZLES.filter((p) => progress.puzzleSolved(p.id)).length);
const firstOpen = computed(() => PUZZLES.find((p) => !progress.puzzleSolved(p.id)));
</script>

<template>
  <div class="page">
    <header class="head">
      <div>
        <p class="eyebrow">Entraînement tactique</p>
        <h1>Puzzles</h1>
        <p class="muted">Une position, un coup gagnant à trouver. {{ solved }} / {{ PUZZLES.length }} résolus.</p>
      </div>
      <button v-if="firstOpen" type="button" class="btn btn-primary" @click="ui.go({ name: 'puzzle', id: firstOpen.id })">
        <Icon name="play" :size="16" /> Puzzle suivant
      </button>
    </header>
    <div class="grid">
      <button
        v-for="(p, i) in PUZZLES"
        :key="p.id"
        type="button"
        class="card glass"
        :class="{ done: progress.puzzleSolved(p.id) }"
        :style="{ animationDelay: `${i * 30}ms` }"
        @click="ui.go({ name: 'puzzle', id: p.id })"
      >
        <div class="mini"><ChessBoard :fen="p.fen" :orientation="p.fen.split(' ')[1] === 'b' ? 'b' : 'w'" :show-coords="false" /></div>
        <div class="meta">
          <span class="chip">{{ p.theme }}</span>
          <h3>{{ i + 1 }}. {{ p.title }}</h3>
          <p class="muted">{{ p.fen.split(" ")[1] === "w" ? "Les Blancs jouent" : "Les Noirs jouent" }}</p>
        </div>
        <span v-if="progress.puzzleSolved(p.id)" class="badge"><Icon name="check" :size="15" /></span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.page {
  height: 100%;
  overflow-y: auto;
  padding: 36px 48px 48px;
}
.head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 24px;
}
h1 {
  font-size: 44px;
  margin: 4px 0 6px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 16px;
}
.card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  text-align: left;
  animation: pop-in 0.4s var(--spring) both;
  transition: transform 0.25s var(--spring), border-color 0.2s;
}
.card:hover {
  transform: translateY(-5px);
  border-color: var(--line-strong);
}
.card.done {
  border-color: color-mix(in srgb, var(--good) 45%, transparent);
}
.mini {
  pointer-events: none;
}
.mini :deep(.board-wrap) {
  --frame-pad: 6px;
  border-radius: 14px;
}
h3 {
  margin-top: 6px;
  font-size: 16px;
}
.meta p {
  margin: 2px 0 0;
  font-size: 12.5px;
}
.badge {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 2;
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: var(--green);
  color: #ffffff;
}
</style>
