<script setup lang="ts">
import { computed, watch } from "vue";
import { setSoundEnabled } from "@/lib/sound";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import Icon from "@/components/Icon.vue";
import PieceDefs from "@/components/PieceDefs.vue";
import ThemeBackground from "@/components/ThemeBackground.vue";
import AppearanceView from "@/views/AppearanceView.vue";
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

// Le thème est porté par <html data-theme="…"> : toute la feuille de styles en dépend.
watch(
  () => progress.settings.theme,
  (theme) => (document.documentElement.dataset.theme = theme),
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
  <ThemeBackground />
  <div class="app">
    <nav class="rail" aria-label="Navigation principale">
      <button type="button" class="brand" title="Chess Academy" @click="ui.go({ name: 'home' })">
        <img src="/logo.png" alt="Chess Academy" />
      </button>
      <button
        v-for="item in NAV"
        :key="item.name"
        type="button"
        class="rail-item"
        :class="{ active: section === item.name }"
        @click="ui.go({ name: item.name })"
      >
        <Icon :name="item.icon" :size="21" />
        <span>{{ item.label }}</span>
      </button>
      <div class="rail-foot">
        <button type="button" class="rail-item" :class="{ active: section === 'appearance' }" @click="ui.go({ name: 'appearance' })">
          <Icon name="palette" :size="21" />
          <span>Apparence</span>
        </button>
        <button
          type="button"
          class="rail-item small"
          :title="progress.settings.sound ? 'Couper le son' : 'Activer le son'"
          @click="progress.setSetting('sound', !progress.settings.sound)"
        >
          <Icon :name="progress.settings.sound ? 'sound' : 'mute'" :size="19" />
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
        <AppearanceView v-else-if="ui.route.name === 'appearance'" key="appearance" />
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
  grid-template-columns: 96px 1fr;
}
.rail {
  --rail-bg: #16130f;
  --rail-ink: rgba(244, 239, 230, 0.55);
  --rail-ink-on: #f4efe6;
  --rail-mark: #c9a96a;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 18px 0 14px;
  background: var(--rail-bg);
  box-shadow: 1px 0 0 rgba(201, 169, 106, 0.25);
}
:global([data-theme="persona5"] .rail) {
  --rail-bg: #0b0b0b;
  --rail-ink: rgba(255, 255, 255, 0.6);
  --rail-ink-on: #ffffff;
  --rail-mark: #e0001b;
  box-shadow: 3px 0 0 #e0001b;
}
:global([data-theme="persona3"] .rail) {
  --rail-bg: rgba(3, 11, 36, 0.92);
  --rail-ink: rgba(200, 225, 255, 0.55);
  --rail-ink-on: #ffffff;
  --rail-mark: #3fe0ff;
  box-shadow: 1px 0 0 rgba(63, 224, 255, 0.3);
}
:global([data-theme="sumi"] .rail) {
  --rail-bg: rgba(233, 222, 199, 0.92);
  --rail-ink: rgba(27, 27, 27, 0.5);
  --rail-ink-on: #1b1b1b;
  --rail-mark: #b8321f;
  box-shadow: 1px 0 0 rgba(27, 27, 27, 0.2);
}
.brand {
  width: 60px;
  height: 60px;
  margin-bottom: 18px;
  padding: 0;
  border: 0;
  background: none;
  transition: transform 0.3s var(--spring);
}
.brand:hover {
  transform: scale(1.05);
}
.brand img {
  width: 100%;
  height: 100%;
  border-radius: 14px;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35);
}
.rail-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
  width: 80px;
  padding: 10px 0 8px;
  border: 0;
  background: none;
  color: var(--rail-ink);
  font-family: var(--font);
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
  transition: color 0.2s, background 0.2s;
}
.rail-item:hover {
  color: var(--rail-ink-on);
}
.rail-item.active {
  color: var(--rail-ink-on);
}
.rail-item.active::before {
  content: "";
  position: absolute;
  left: -8px;
  top: 20%;
  bottom: 20%;
  width: 3px;
  background: var(--rail-mark);
}
:global([data-theme="persona5"] .rail-item.active) {
  background: #e0001b;
  clip-path: polygon(0 8%, 100% 0, 94% 100%, 4% 92%);
}
:global([data-theme="persona5"] .rail-item.active::before) {
  display: none;
}
:global([data-theme="persona3"] .rail-item.active) {
  background: #fff;
  color: #061640;
  clip-path: polygon(8px 0, 100% 0, calc(100% - 8px) 100%, 0 100%);
}
:global([data-theme="persona3"] .rail-item.active::before) {
  display: none;
}
:global([data-theme="sumi"] .rail-item.active::before) {
  left: auto;
  right: 6px;
  top: 10px;
  bottom: auto;
  width: 8px;
  height: 8px;
  border-radius: 1px;
  transform: rotate(8deg);
}
.rail-item.small {
  width: 48px;
  padding: 10px 0;
}
.rail-foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
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
  transform: translateY(12px);
}
.page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
.toast {
  position: fixed;
  left: 50%;
  bottom: 30px;
  z-index: 90;
  transform: translateX(-50%);
  padding: 12px 22px;
  border-radius: var(--radius);
  background: var(--surface-solid);
  color: var(--ink);
  box-shadow: 0 0 0 1px var(--line-strong), var(--shadow);
  font-weight: 600;
}
.toast.win {
  background: var(--accent);
  color: var(--accent-ink);
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
