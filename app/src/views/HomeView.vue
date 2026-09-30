<script setup lang="ts">
import { computed } from "vue";
import { START_FEN } from "@/lib/chess";
import { LESSONS, PUZZLES, TIPS } from "@/lib/content";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import ChessBoard from "@/components/ChessBoard.vue";
import Icon from "@/components/Icon.vue";
import Mascot from "@/components/Mascot.vue";

const ui = useUiStore();
const progress = useProgressStore();

const doneCount = computed(() => LESSONS.filter((l) => progress.lessonDone(l.id)).length);
const nextLesson = computed(() => LESSONS.find((l) => !progress.lessonDone(l.id)) ?? null);
const solved = computed(() => PUZZLES.filter((p) => progress.puzzleSolved(p.id)).length);
const tip = TIPS[Math.floor(Date.now() / 86_400_000) % TIPS.length];

const greeting = computed(() => {
  if (doneCount.value === 0) return "Bienvenue. Je suis Academy, ton professeur. Nous apprendrons les échecs pas à pas, des premières règles jusqu'aux finales.";
  if (!nextLesson.value) return "Tu as terminé toutes mes leçons. Il est temps de me défier en partie.";
  return `Content de te revoir. Prochaine leçon : « ${nextLesson.value.title} ».`;
});

const MODES = [
  { name: "lessons", num: "I", title: "Leçons", icon: "book", desc: "Règles, pièces, tactiques et finales, avec des exercices sur l'échiquier." },
  { name: "play", num: "II", title: "Partie commentée", icon: "swords", desc: "Joue contre Academy : chaque coup est analysé et expliqué." },
  { name: "puzzles", num: "III", title: "Puzzles", icon: "puzzle", desc: "Trouve le coup gagnant : mats, fourchettes, clouages…" },
  { name: "coords", num: "IV", title: "Les cases", icon: "target", desc: "Apprends le nom des 64 cases contre la montre." },
] as const;
</script>

<template>
  <div class="home">
    <section class="hero">
      <div class="hero-text">
        <p class="eyebrow">Académie d'échecs · Ton professeur personnel</p>
        <h1 class="title">
          <span class="t1">Chess</span>
          <span class="t2">Academy</span>
        </h1>
        <div class="rule" />
        <div class="coach">
          <Mascot :size="64" :mood="doneCount ? 'happy' : 'wow'" />
          <p class="quote">{{ greeting }}</p>
        </div>
        <div class="cta">
          <button v-if="nextLesson" type="button" class="btn btn-primary" @click="ui.go({ name: 'lesson', id: nextLesson.id })">
            <Icon name="play" :size="14" />
            {{ doneCount ? "Continuer" : "Commencer" }}
          </button>
          <button type="button" class="btn btn-ghost" @click="ui.go({ name: 'play' })">Jouer une partie</button>
        </div>
        <dl class="stats">
          <div>
            <dt>Leçons</dt>
            <dd>{{ doneCount }}<small>/{{ LESSONS.length }}</small></dd>
          </div>
          <div>
            <dt>Puzzles</dt>
            <dd>{{ solved }}<small>/{{ PUZZLES.length }}</small></dd>
          </div>
          <div>
            <dt>Victoires</dt>
            <dd>{{ progress.wins }}</dd>
          </div>
          <div>
            <dt>Record cases</dt>
            <dd>{{ progress.coordsBest.find ?? 0 }}</dd>
          </div>
        </dl>
      </div>
      <div class="hero-board" aria-hidden="true">
        <div class="stage">
          <ChessBoard :fen="START_FEN" :show-coords="true" />
        </div>
      </div>
    </section>

    <section class="modes">
      <button v-for="(m, i) in MODES" :key="m.name" type="button" class="mode" :style="{ animationDelay: `${i * 70}ms` }" @click="ui.go({ name: m.name })">
        <span class="num">{{ m.num }}</span>
        <span class="mode-body">
          <strong>{{ m.title }}</strong>
          <small>{{ m.desc }}</small>
        </span>
        <Icon name="next" :size="18" class="go" />
      </button>
    </section>

    <section class="tip">
      <span class="tip-label">Conseil du jour</span>
      <p>{{ tip }}</p>
    </section>
  </div>
</template>

<style scoped>
.home {
  height: 100%;
  overflow-y: auto;
  padding: 44px 56px 44px;
}
.hero {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(0, 0.95fr);
  gap: 56px;
  align-items: center;
}
.title {
  display: flex;
  flex-direction: column;
  margin: 14px 0 0;
  font-size: clamp(56px, 7vw, 104px);
  line-height: 0.92;
}
.t2 {
  font-style: italic;
  color: var(--accent-2);
}
.rule {
  width: 96px;
  height: 2px;
  margin: 26px 0 24px;
  background: var(--accent-2);
}
.coach {
  display: flex;
  align-items: center;
  gap: 16px;
  max-width: 560px;
}
.quote {
  margin: 0;
  font-family: var(--font-display);
  font-size: 21px;
  font-style: italic;
  line-height: 1.45;
  color: var(--ink);
}
.cta {
  display: flex;
  gap: 12px;
  margin-top: 30px;
}
.stats {
  display: flex;
  margin: 38px 0 0;
  border-top: 1px solid var(--line-strong);
}
.stats > div {
  flex: 1;
  padding: 14px 16px 0 0;
}
.stats > div + div {
  padding-left: 16px;
  border-left: 1px solid var(--line);
}
dt {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-3);
}
dd {
  margin: 4px 0 0;
  font-family: var(--font-display);
  font-size: 38px;
  line-height: 1;
}
dd small {
  font-size: 18px;
  color: var(--ink-3);
}
.hero-board {
  display: grid;
  place-items: center;
}
.stage {
  width: min(100%, 520px);
  animation: settle 0.9s var(--ease) both;
}
@keyframes settle {
  from {
    opacity: 0;
    transform: translateY(18px) scale(0.98);
  }
}

.modes {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  margin-top: 52px;
  border-top: 1px solid var(--line-strong);
  border-bottom: 1px solid var(--line-strong);
}
.mode {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 14px;
  align-items: start;
  padding: 22px 20px;
  border: 0;
  background: none;
  text-align: left;
  animation: pop-in 0.5s var(--ease) both;
  transition: background 0.25s;
}
.mode + .mode {
  border-left: 1px solid var(--line);
}
.mode:hover {
  background: var(--surface);
}
.num {
  font-family: var(--font-display);
  font-size: 30px;
  line-height: 1;
  color: var(--accent-2);
}
.mode-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mode strong {
  font-family: var(--font-display);
  font-size: 22px;
  font-weight: var(--display-weight);
  font-style: var(--display-style);
  text-transform: var(--display-transform);
  line-height: 1.1;
}
.mode small {
  font-size: 13px;
  line-height: 1.5;
  color: var(--ink-2);
}
.go {
  margin-top: 4px;
  color: var(--ink-3);
  transition: transform 0.2s, color 0.2s;
}
.mode:hover .go {
  color: var(--accent);
  transform: translateX(3px);
}
.tip {
  display: flex;
  align-items: baseline;
  gap: 18px;
  margin-top: 26px;
}
.tip-label {
  flex: none;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--accent-2);
}
.tip p {
  margin: 0;
  font-family: var(--font-display);
  font-size: 19px;
  font-style: italic;
}

/* ---- variantes de thème */
:global([data-theme="persona5"] .home .title .t1) {
  color: #fff;
  -webkit-text-stroke: 0;
}
:global([data-theme="persona5"] .home .title .t2) {
  display: inline-block;
  align-self: flex-start;
  padding: 0 18px;
  font-style: normal;
  color: #0b0b0b;
  background: #fff;
  transform: rotate(-4deg) skewX(-8deg);
  box-shadow: 8px 8px 0 #e0001b;
}
:global([data-theme="persona5"] .home .rule),
:global([data-theme="persona3"] .home .rule) {
  height: 4px;
  transform: skewX(-30deg);
}
:global([data-theme="persona5"] .home .quote),
:global([data-theme="persona3"] .home .quote),
:global([data-theme="persona5"] .home .tip p),
:global([data-theme="persona3"] .home .tip p) {
  font-family: var(--font);
  font-style: normal;
  font-size: 17px;
  font-weight: 600;
}
:global([data-theme="persona5"] .home .mode:hover) {
  background: #e0001b;
}
:global([data-theme="persona3"] .home .title .t2) {
  color: var(--accent-2);
  text-shadow: 0 0 30px rgba(63, 224, 255, 0.45);
}
:global([data-theme="sumi"] .home .title) {
  font-family: "Yuji Syuku", serif;
  font-weight: 400;
}
:global([data-theme="sumi"] .home .title .t2) {
  font-style: normal;
  color: var(--accent);
}
:global([data-theme="sumi"] .home .rule) {
  height: 6px;
  width: 140px;
  border-radius: 3px;
  background: #1b1b1b;
  -webkit-mask-image: linear-gradient(90deg, #000 60%, transparent);
  mask-image: linear-gradient(90deg, #000 60%, transparent);
}
:global([data-theme="sumi"] .home .mode) {
  background: rgba(247, 241, 227, 0.72);
}
:global([data-theme="sumi"] .home .num) {
  font-family: "Yuji Syuku", serif;
  color: var(--accent);
}

@media (max-width: 1150px) {
  .hero {
    grid-template-columns: 1fr;
  }
  .modes {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
