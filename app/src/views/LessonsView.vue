<script setup lang="ts">
import { computed } from "vue";
import { CHAPTERS, LESSONS } from "@/lib/content";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import Icon from "@/components/Icon.vue";
import LessonIcon from "@/components/LessonIcon.vue";

const ui = useUiStore();
const progress = useProgressStore();

const groups = computed(() =>
  CHAPTERS.map((chapter, i) => {
    const lessons = LESSONS.filter((l) => l.chapter === chapter);
    return { chapter, index: i + 1, lessons, done: lessons.filter((l) => progress.lessonDone(l.id)).length };
  }),
);
const total = computed(() => LESSONS.filter((l) => progress.lessonDone(l.id)).length);
const nextId = computed(() => LESSONS.find((l) => !progress.lessonDone(l.id))?.id);
</script>

<template>
  <div class="page">
    <header class="head">
      <div>
        <p class="eyebrow">Le parcours</p>
        <h1>Leçons</h1>
        <p class="muted">Des bases jusqu'aux finales. Chaque leçon mêle explications et exercices sur l'échiquier.</p>
      </div>
      <div class="ring" :style="{ '--p': total / LESSONS.length }">
        <strong>{{ Math.round((total / LESSONS.length) * 100) }} %</strong>
        <span>{{ total }} / {{ LESSONS.length }}</span>
      </div>
    </header>

    <section v-for="g in groups" :key="g.chapter" class="chapter">
      <h2>
        <span class="num">{{ g.index }}</span>
        {{ g.chapter }}
        <small>{{ g.done }}/{{ g.lessons.length }}</small>
      </h2>
      <div class="grid">
        <button
          v-for="l in g.lessons"
          :key="l.id"
          type="button"
          class="card glass"
          :class="{ done: progress.lessonDone(l.id), next: l.id === nextId }"
          @click="ui.go({ name: 'lesson', id: l.id })"
        >
          <LessonIcon :icon="l.icon" />
          <div class="info">
            <h3>{{ l.title }}</h3>
            <p>{{ l.summary }}</p>
          </div>
          <span v-if="progress.lessonDone(l.id)" class="badge"><Icon name="check" :size="16" /></span>
          <span v-else-if="l.id === nextId" class="chip next-chip">À suivre</span>
        </button>
      </div>
    </section>
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
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 10px;
}
h1 {
  font-size: 44px;
  margin: 4px 0 6px;
}
.ring {
  --p: 0;
  display: grid;
  place-content: center;
  text-align: center;
  width: 116px;
  height: 116px;
  border-radius: 50%;
  background:
    radial-gradient(circle, var(--bg-1) 60%, transparent 61%),
    conic-gradient(var(--pink), var(--cyan) calc(var(--p) * 360deg), var(--surface-2) 0);
  box-shadow: 0 0 30px color-mix(in srgb, var(--accent) 25%, transparent);
}
.ring strong {
  font-family: var(--font-display);
  font-size: 24px;
}
.ring span {
  font-size: 12px;
  color: var(--text-2);
}
.chapter {
  margin-top: 28px;
}
h2 {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
  font-size: 22px;
}
h2 small {
  font-family: var(--font);
  font-size: 13px;
  color: var(--text-3);
}
.num {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  background: var(--grad);
  font-size: 16px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}
.card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  text-align: left;
  transition: transform 0.25s var(--spring), box-shadow 0.25s, border-color 0.25s;
}
.card:hover {
  transform: translateY(-4px);
  border-color: var(--line-strong);
}
.card.done {
  border-color: color-mix(in srgb, var(--good) 40%, transparent);
}
.card.next {
  border-color: color-mix(in srgb, var(--accent) 60%, transparent);
  box-shadow: var(--shadow), 0 0 24px color-mix(in srgb, var(--accent) 25%, transparent);
}
.info {
  min-width: 0;
}
h3 {
  font-size: 17px;
}
.info p {
  margin: 2px 0 0;
  font-size: 13px;
  color: var(--text-2);
  line-height: 1.4;
}
.badge {
  position: absolute;
  top: 10px;
  right: 10px;
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--green);
  color: #ffffff;
}
.next-chip {
  position: absolute;
  top: 10px;
  right: 10px;
  background: color-mix(in srgb, var(--accent) 25%, transparent);
  color: var(--ink);
}
</style>
