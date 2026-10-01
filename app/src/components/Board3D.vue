<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { START_FEN } from "@/lib/chess";
import { DemoGames, type DemoStatus } from "@/lib/board3d/demo";
import { BoardScene } from "@/lib/board3d/scene";
import { useProgressStore } from "@/stores/progress";
import ChessBoard from "./ChessBoard.vue";

// Plateau 3D de l'accueil : rotation permanente, retournement complet de temps en temps,
// manipulation libre, et partie toujours en cours. Sans 3D disponible (WebGL), le
// plateau 2D habituel s'affiche à la place.
const progress = useProgressStore();
const wrap = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const failed = ref(false);
const status = ref<DemoStatus | null>(null);

let scene: BoardScene | null = null;
let demo: DemoGames | null = null;
let resize: ResizeObserver | null = null;
let seen: IntersectionObserver | null = null;
let onScreen = true;
let waiters: (() => void)[] = [];

const visible = () => onScreen && !document.hidden;

function update() {
  if (!scene) return;
  if (visible()) {
    scene.start();
    waiters.forEach((w) => w());
    waiters = [];
  } else {
    scene.stop();
  }
}

function waitVisible(): Promise<void> {
  return visible() ? Promise.resolve() : new Promise((r) => waiters.push(r));
}

onMounted(() => {
  try {
    scene = new BoardScene(canvas.value!);
  } catch {
    failed.value = true;
    return;
  }
  scene.setTheme(progress.settings.board, progress.settings.pieces);
  resize = new ResizeObserver(([entry]) => scene?.resize(entry.contentRect.width, entry.contentRect.height));
  resize.observe(canvas.value!);
  seen = new IntersectionObserver(([entry]) => {
    onScreen = entry.isIntersecting;
    update();
  });
  seen.observe(wrap.value!);
  document.addEventListener("visibilitychange", update);
  update();
  demo = new DemoGames(scene, (s) => (status.value = s), waitVisible);
  void demo.run();
});

watch(
  () => [progress.settings.board, progress.settings.pieces] as const,
  ([board, pieces]) => scene?.setTheme(board, pieces),
);

onBeforeUnmount(() => {
  demo?.stop();
  waiters.forEach((w) => w());
  waiters = [];
  resize?.disconnect();
  seen?.disconnect();
  document.removeEventListener("visibilitychange", update);
  scene?.dispose();
  scene = null;
});
</script>

<template>
  <div ref="wrap" class="board3d">
    <ChessBoard v-if="failed" :fen="START_FEN" :show-coords="true" />
    <template v-else>
      <canvas ref="canvas" class="view" aria-label="Échiquier en 3D : une partie se joue, faites-le tourner à la souris ou au doigt" />
      <Transition name="caption" mode="out-in">
        <div v-if="status" :key="status.title" class="caption">
          <span class="kind">{{ status.kind === "famous" ? "Partie célèbre" : "Partie du moteur" }}</span>
          <strong>{{ status.title }}</strong>
          <span class="sub">{{ status.subtitle }}</span>
        </div>
      </Transition>
      <p v-if="status" class="move">{{ status.move }}</p>
    </template>
  </div>
</template>

<style scoped>
.board3d {
  display: grid;
  justify-items: center;
  gap: 6px;
  width: 100%;
}
.view {
  display: block;
  width: 100%;
  aspect-ratio: 1;
  cursor: grab;
  touch-action: none;
}
.view:active {
  cursor: grabbing;
}
.caption {
  display: grid;
  justify-items: center;
  gap: 2px;
  text-align: center;
}
.kind {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--accent);
}
.caption strong {
  font-family: var(--font-display, inherit);
  font-size: 18px;
}
.sub {
  font-size: 12.5px;
  color: var(--ink-3);
}
.move {
  margin: 2px 0 0;
  font-size: 15px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
.caption-enter-active,
.caption-leave-active {
  transition:
    opacity 0.35s ease,
    transform 0.35s ease;
}
.caption-enter-from,
.caption-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>
