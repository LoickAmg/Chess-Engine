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
  if (doneCount.value === 0) return "Salut ! Je suis Lumi. Je vais t'apprendre les échecs, pas à pas. Prêt(e) pour ta première leçon ?";
  if (!nextLesson.value) return "Tu as terminé toutes mes leçons ! Il est temps de me défier en partie.";
  return `Content de te revoir ! Prochaine étape : « ${nextLesson.value.title} ».`;
});

const MODES = [
  { name: "lessons", title: "Leçons", icon: "book", desc: "Les règles, les pièces, les tactiques et les finales, avec des exercices.", tone: "pink" },
  { name: "play", title: "Jouer contre Lumi", icon: "swords", desc: "Une vraie partie : je commente tes coups et je t'aide si tu veux.", tone: "violet" },
  { name: "puzzles", title: "Puzzles", icon: "puzzle", desc: "Trouve le coup gagnant : mats, fourchettes, clouages…", tone: "cyan" },
  { name: "coords", title: "Entraînement aux cases", icon: "target", desc: "Apprends le nom des cases contre la montre.", tone: "gold" },
] as const;
</script>

<template>
  <div class="home">
    <section class="hero">
      <div class="hero-text">
        <p class="eyebrow">Ton professeur d'échecs</p>
        <h1>Chess <span class="grad-text">Académie</span></h1>
        <div class="coach">
          <Mascot :size="92" :mood="doneCount ? 'happy' : 'wow'" />
          <p class="bubble">{{ greeting }}</p>
        </div>
        <div class="cta">
          <button v-if="nextLesson" type="button" class="btn btn-primary" @click="ui.go({ name: 'lesson', id: nextLesson.id })">
            <Icon name="play" :size="16" />
            {{ doneCount ? "Continuer" : "Commencer" }} : {{ nextLesson.title }}
          </button>
          <button type="button" class="btn btn-ghost" @click="ui.go({ name: 'play' })">
            <Icon name="swords" :size="18" /> Jouer une partie
          </button>
        </div>
        <div class="stats">
          <div class="stat">
            <strong>{{ doneCount }}<small>/{{ LESSONS.length }}</small></strong>
            <span>leçons</span>
          </div>
          <div class="stat">
            <strong>{{ solved }}<small>/{{ PUZZLES.length }}</small></strong>
            <span>puzzles</span>
          </div>
          <div class="stat">
            <strong>{{ progress.wins }}</strong>
            <span>victoires</span>
          </div>
          <div class="stat">
            <strong>{{ progress.coordsBest.find ?? 0 }}</strong>
            <span>record cases</span>
          </div>
        </div>
      </div>
      <div class="hero-board" aria-hidden="true">
        <div class="tilt">
          <ChessBoard :fen="START_FEN" :show-coords="false" />
        </div>
      </div>
    </section>

    <section class="modes">
      <button
        v-for="(m, i) in MODES"
        :key="m.name"
        type="button"
        class="mode glass"
        :class="m.tone"
        :style="{ animationDelay: `${i * 70}ms` }"
        @click="ui.go({ name: m.name })"
      >
        <span class="mode-icon"><Icon :name="m.icon" :size="26" /></span>
        <h3>{{ m.title }}</h3>
        <p>{{ m.desc }}</p>
        <span class="go"><Icon name="next" :size="18" /></span>
      </button>
    </section>

    <section class="tip glass">
      <Icon name="bulb" :size="22" />
      <p><strong>Astuce du jour :</strong> {{ tip }}</p>
    </section>
  </div>
</template>

<style scoped>
.home {
  height: 100%;
  overflow-y: auto;
  padding: 36px 48px 40px;
}
.hero {
  display: grid;
  grid-template-columns: 1.15fr 0.85fr;
  gap: 40px;
  align-items: center;
}
h1 {
  margin: 6px 0 18px;
  font-size: clamp(44px, 5.2vw, 72px);
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.01em;
  text-shadow: 0 6px 30px rgba(255, 79, 163, 0.25);
}
.coach {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 22px;
}
.bubble {
  position: relative;
  margin: 0;
  max-width: 460px;
  padding: 14px 18px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.95);
  color: #2c1263;
  font-weight: 700;
  line-height: 1.45;
  box-shadow: 0 10px 30px rgba(10, 0, 30, 0.45);
  animation: pop-in 0.5s var(--spring) 0.15s both;
}
.bubble::before {
  content: "";
  position: absolute;
  left: -8px;
  top: 50%;
  width: 16px;
  height: 16px;
  background: inherit;
  transform: translateY(-50%) rotate(45deg);
  border-radius: 3px;
}
.cta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.stats {
  display: flex;
  gap: 12px;
  margin-top: 26px;
}
.stat {
  display: flex;
  flex-direction: column;
  min-width: 96px;
  padding: 10px 16px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px var(--line);
}
.stat strong {
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 600;
  line-height: 1.1;
}
.stat small {
  font-size: 15px;
  color: var(--text-3);
}
.stat span {
  font-size: 12.5px;
  color: var(--text-2);
}
.hero-board {
  perspective: 1400px;
  display: grid;
  place-items: center;
}
.tilt {
  width: min(100%, 420px);
  transform: rotateX(24deg) rotateZ(-8deg);
  transform-style: preserve-3d;
  filter: drop-shadow(0 40px 40px rgba(0, 0, 0, 0.45));
  animation: showcase 9s ease-in-out infinite;
}
@keyframes showcase {
  50% {
    transform: rotateX(18deg) rotateZ(-3deg) translateY(-10px);
  }
}
.modes {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-top: 34px;
}
.mode {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  min-height: 190px;
  padding: 20px;
  text-align: left;
  overflow: hidden;
  animation: pop-in 0.5s var(--spring) both;
  transition: transform 0.25s var(--spring), box-shadow 0.25s;
}
.mode:hover {
  transform: translateY(-6px);
}
.mode::after {
  content: "";
  position: absolute;
  right: -40px;
  top: -40px;
  width: 140px;
  height: 140px;
  border-radius: 50%;
  background: var(--tone);
  opacity: 0.2;
  filter: blur(20px);
  transition: opacity 0.3s;
}
.mode:hover::after {
  opacity: 0.4;
}
.mode.pink {
  --tone: var(--pink);
}
.mode.violet {
  --tone: var(--violet);
}
.mode.cyan {
  --tone: var(--cyan);
}
.mode.gold {
  --tone: var(--gold);
}
.mode-icon {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  margin-bottom: 8px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--tone) 22%, transparent);
  color: var(--tone);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--tone) 45%, transparent);
}
.mode h3 {
  font-size: 19px;
}
.mode p {
  margin: 0;
  font-size: 13.5px;
  color: var(--text-2);
  line-height: 1.45;
}
.go {
  margin-top: auto;
  align-self: flex-end;
  color: var(--tone);
}
.tip {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-top: 18px;
  padding: 16px 20px;
  color: var(--gold);
}
.tip p {
  margin: 0;
  color: var(--text);
}
.tip strong {
  color: var(--gold);
}
@media (max-width: 1150px) {
  .modes {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
