<script setup lang="ts">
import { computed, watch } from "vue";
import { setSoundEnabled } from "@/lib/sound";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import CosmosBackground from "@/components/CosmosBackground.vue";
import Icon from "@/components/Icon.vue";
import PieceDefs from "@/components/PieceDefs.vue";
import CoordsView from "@/views/CoordsView.vue";
import GlossaryView from "@/views/GlossaryView.vue";
import HomeView from "@/views/HomeView.vue";
import LessonView from "@/views/LessonView.vue";
import LessonsView from "@/views/LessonsView.vue";
import PlayView from "@/views/PlayView.vue";
import PuzzleView from "@/views/PuzzleView.vue";
import PuzzlesView from "@/views/PuzzlesView.vue";

const ui = useUiStore();
const progress = useProgressStore();

watch(
  () => progress.settings.sound,
  (on) => setSoundEnabled(on),
  { immediate: true },
);

const NAV = [
  { name: "home", label: "Accueil", icon: "home" },
  { name: "lessons", label: "Leçons", icon: "book" },
  { name: "play", label: "Jouer", icon: "swords" },
  { name: "puzzles", label: "Puzzles", icon: "puzzle" },
  { name: "coords", label: "Cases", icon: "target" },
  { name: "glossary", label: "Lexique", icon: "library" },
] as const;

const section = computed(() => {
  const n = ui.route.name;
  if (n === "lesson") return "lessons";
  if (n === "puzzle") return "puzzles";
  return n;
});
</script>

<template>
  <PieceDefs />
  <CosmosBackground />
  <div class="app">
    <nav class="rail" aria-label="Navigation principale">
      <button type="button" class="brand" title="Accueil" @click="ui.go({ name: 'home' })">
        <img src="/logo.png" alt="" />
      </button>
      <button
        v-for="item in NAV"
        :key="item.name"
        type="button"
        class="rail-item"
        :class="{ active: section === item.name }"
        @click="ui.go({ name: item.name })"
      >
        <Icon :name="item.icon" :size="22" />
        <span>{{ item.label }}</span>
      </button>
      <div class="rail-foot">
        <button
          type="button"
          class="rail-item small"
          :title="progress.settings.sound ? 'Couper le son' : 'Activer le son'"
          @click="progress.setSetting('sound', !progress.settings.sound)"
        >
          <Icon :name="progress.settings.sound ? 'sound' : 'mute'" :size="20" />
        </button>
      </div>
    </nav>

    <main class="main">
      <Transition name="page" mode="out-in">
        <HomeView v-if="ui.route.name === 'home'" key="home" />
        <LessonsView v-else-if="ui.route.name === 'lessons'" key="lessons" />
        <LessonView v-else-if="ui.route.name === 'lesson'" :id="ui.route.id" :key="`lesson-${ui.route.id}`" />
        <PlayView v-else-if="ui.route.name === 'play'" key="play" />
        <PuzzlesView v-else-if="ui.route.name === 'puzzles'" key="puzzles" />
        <PuzzleView v-else-if="ui.route.name === 'puzzle'" :id="ui.route.id" :key="`puzzle-${ui.route.id}`" />
        <CoordsView v-else-if="ui.route.name === 'coords'" key="coords" />
        <GlossaryView v-else-if="ui.route.name === 'glossary'" key="glossary" />
      </Transition>
    </main>

    <Transition name="toast">
      <div v-if="ui.toast" class="toast" :class="ui.toast.kind" role="status">{{ ui.toast.text }}</div>
    </Transition>
  </div>
</template>

<style scoped>
.app {
  position: relative;
  z-index: 1;
  height: 100%;
  display: grid;
  grid-template-columns: 92px 1fr;
}
.rail {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 16px 0 14px;
  background: linear-gradient(180deg, rgba(20, 6, 52, 0.85), rgba(10, 2, 30, 0.9));
  border-right: 1px solid var(--line);
  backdrop-filter: blur(16px);
}
.brand {
  width: 56px;
  height: 56px;
  margin-bottom: 14px;
  padding: 0;
  border: 0;
  background: none;
  transition: transform 0.3s var(--spring);
}
.brand:hover {
  transform: rotate(-6deg) scale(1.06);
}
.brand img {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 6px 14px rgba(255, 79, 163, 0.35));
}
.rail-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 74px;
  padding: 10px 0 8px;
  border: 0;
  border-radius: 16px;
  background: none;
  color: var(--text-3);
  font-family: var(--font-display);
  font-size: 12.5px;
  font-weight: 500;
  transition: color 0.2s, background 0.2s, transform 0.2s var(--spring);
}
.rail-item:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.05);
}
.rail-item.active {
  color: #fff;
  background: linear-gradient(160deg, rgba(255, 79, 163, 0.28), rgba(50, 224, 255, 0.16));
  box-shadow: inset 0 0 0 1px rgba(255, 150, 210, 0.35), 0 6px 20px rgba(255, 79, 163, 0.18);
}
.rail-item.active::before {
  content: "";
  position: absolute;
  left: -9px;
  top: 22%;
  bottom: 22%;
  width: 4px;
  border-radius: 4px;
  background: var(--grad);
}
.rail-item.small {
  width: 48px;
  padding: 10px 0;
}
.rail-foot {
  margin-top: auto;
}
.main {
  position: relative;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.page-enter-active,
.page-leave-active {
  transition: opacity 0.22s, transform 0.3s var(--ease);
}
.page-enter-from {
  opacity: 0;
  transform: translateY(14px);
}
.page-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
.toast {
  position: fixed;
  left: 50%;
  bottom: 30px;
  z-index: 90;
  transform: translateX(-50%);
  padding: 12px 22px;
  border-radius: 14px;
  background: var(--panel-strong);
  border: 1px solid var(--line-strong);
  box-shadow: var(--shadow);
  font-family: var(--font-display);
  font-weight: 600;
}
.toast.win {
  background: linear-gradient(120deg, rgba(255, 79, 163, 0.95), rgba(155, 107, 255, 0.95));
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.25s, transform 0.35s var(--spring);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 16px);
}
</style>
