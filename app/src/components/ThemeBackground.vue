<script setup lang="ts">
import { computed } from "vue";
import { useProgressStore } from "@/stores/progress";

// Décor de fond propre à chaque thème (purement décoratif).
const progress = useProgressStore();
const theme = computed(() => progress.settings.theme);
</script>

<template>
  <div class="bg" :class="`t-${theme}`" aria-hidden="true">
    <!-- Classique : papier ivoire, lumière douce, filigrane d'un cavalier gravé -->
    <template v-if="theme === 'classic'">
      <div class="paper" />
      <svg class="engraving" viewBox="0 0 100 100">
        <path
          d="M33.5 80.5 C33.5 72 36.5 66 42.5 60.5 C37 60.5 31 62 27 59.5 C22.5 56.5 21.5 51 24.5 47 C28.5 42 34.5 39 38.5 35 C40.5 28 40.5 21.5 45 14 C47.5 17 49 19.5 50.5 22 C57.5 18.5 66.5 20.5 71.5 27.5 C77.5 36 78.5 48.5 74.5 58.5 C71.5 66 68.5 72.5 68.5 80.5 Z M21 94 C21 88.5 24.5 86 30 86 L70 86 C75.5 86 79 88.5 79 94 Z"
        />
      </svg>
      <div class="rule r1" />
      <div class="rule r2" />
    </template>

    <!-- Persona 5 : éclats rouges et noirs, trame, étoiles -->
    <template v-else-if="theme === 'persona5'">
      <div class="shards" />
      <div class="halftone" />
      <svg class="stars" viewBox="0 0 100 100" preserveAspectRatio="none">
        <polygon points="8,12 10,17 15,17 11,20 13,25 8,22 3,25 5,20 1,17 6,17" />
        <polygon points="88,70 90,75 95,75 91,78 93,83 88,80 83,83 85,78 81,75 86,75" />
        <polygon points="70,8 71,11 74,11 72,13 73,16 70,14 67,16 68,13 66,11 69,11" />
      </svg>
    </template>

    <!-- Persona 3 : eau sombre, rais de lumière, particules -->
    <template v-else-if="theme === 'persona3'">
      <div class="moon" />
      <div class="shafts" />
      <div class="water" />
    </template>

    <!-- Encre de Chine : papier washi, montagnes au lavis, soleil vermillon, calligraphie -->
    <template v-else>
      <div class="washi" />
      <div class="sun" />
      <svg class="mountains" viewBox="0 0 1200 400" preserveAspectRatio="none">
        <defs>
          <linearGradient id="ink-far" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#1b1b1b" stop-opacity="0.18" />
            <stop offset="1" stop-color="#1b1b1b" stop-opacity="0" />
          </linearGradient>
          <linearGradient id="ink-near" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stop-color="#1b1b1b" stop-opacity="0.42" />
            <stop offset="0.7" stop-color="#1b1b1b" stop-opacity="0.08" />
            <stop offset="1" stop-color="#1b1b1b" stop-opacity="0" />
          </linearGradient>
          <filter id="ink-blur"><feGaussianBlur stdDeviation="3" /></filter>
          <filter id="ink-rough">
            <feTurbulence type="fractalNoise" baseFrequency="0.015 0.06" numOctaves="3" seed="3" />
            <feDisplacementMap in="SourceGraphic" scale="14" />
          </filter>
        </defs>
        <path
          d="M0 260 C80 200 140 150 210 170 C270 185 300 120 360 110 C430 100 470 190 540 180 C620 170 660 90 740 100 C820 110 850 200 930 190 C1010 180 1060 130 1200 150 L1200 400 L0 400 Z"
          fill="url(#ink-far)"
          filter="url(#ink-blur)"
        />
        <path
          d="M0 320 C60 280 110 220 180 240 C240 255 280 300 340 290 C420 275 450 200 520 215 C600 230 640 300 720 305 C800 310 850 240 930 250 C1020 262 1080 320 1200 300 L1200 400 L0 400 Z"
          fill="url(#ink-near)"
          filter="url(#ink-rough)"
        />
      </svg>
      <div class="calligraphy">棋道</div>
    </template>
  </div>
</template>

<style scoped>
.bg {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
  background: var(--bg);
}

/* ---- Classique */
.t-classic {
  background: radial-gradient(ellipse 90% 70% at 70% 20%, #fbf8f1 0%, #f1e9dc 55%, #e4d8c3 100%);
}
.paper {
  position: absolute;
  inset: 0;
  opacity: 0.55;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='260' height='260'%3E%3Cfilter id='p'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3'/%3E%3CfeColorMatrix values='0 0 0 0 0.35 0 0 0 0 0.27 0 0 0 0 0.18 0 0 0 0.07 0'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23p)'/%3E%3C/svg%3E");
}
.engraving {
  position: absolute;
  right: -6vw;
  bottom: -8vh;
  width: 58vh;
  height: 58vh;
  fill: none;
  stroke: rgba(120, 90, 50, 0.1);
  stroke-width: 0.35;
}
.rule {
  position: absolute;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(154, 115, 54, 0.3), transparent);
}
.r1 {
  top: 12vh;
}
.r2 {
  bottom: 10vh;
}

/* ---- Persona 5 */
.t-persona5 {
  background: #0b0b0b;
}
.shards {
  position: absolute;
  inset: -10%;
  background:
    linear-gradient(115deg, transparent 0 38%, #b3001a 38% 52%, transparent 52%),
    linear-gradient(-65deg, transparent 0 60%, #e0001b 60% 64%, transparent 64%),
    linear-gradient(160deg, transparent 0 78%, #1f1f1f 78% 90%, transparent 90%);
  animation: jitter 6s steps(2) infinite;
}
@keyframes jitter {
  50% {
    transform: translate(4px, -3px);
  }
}
.halftone {
  position: absolute;
  inset: 0;
  background-image: radial-gradient(circle, rgba(255, 255, 255, 0.08) 22%, transparent 24%);
  background-size: 14px 14px;
  mask-image: linear-gradient(120deg, #000, transparent 70%);
}
.stars {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  fill: #fff;
  opacity: 0.22;
}

/* ---- Persona 3 */
.t-persona3 {
  background: linear-gradient(170deg, #0a2361 0%, #061640 45%, #020716 100%);
}
.moon {
  position: absolute;
  right: 12vw;
  top: 8vh;
  width: 18vh;
  height: 18vh;
  border-radius: 50%;
  background: radial-gradient(circle at 40% 40%, #eaf4ff, #9ec8ff 60%, transparent 71%);
  box-shadow: 0 0 80px 20px rgba(63, 224, 255, 0.2);
  opacity: 0.65;
}
.shafts {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(115deg, transparent 0 80px, rgba(120, 200, 255, 0.05) 80px 86px, transparent 86px 170px);
  animation: drift 20s linear infinite;
}
@keyframes drift {
  to {
    background-position: 400px 0;
  }
}
.water {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 30vh;
  background:
    repeating-linear-gradient(0deg, rgba(63, 224, 255, 0.07) 0 1px, transparent 1px 9px),
    linear-gradient(transparent, rgba(31, 107, 255, 0.18));
  mask-image: linear-gradient(transparent, #000);
}

/* ---- Encre de Chine */
.t-sumi {
  background: #efe6d2;
}
.washi {
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='320'%3E%3Cfilter id='w'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.6 0.04' numOctaves='3' seed='2'/%3E%3CfeColorMatrix values='0 0 0 0 0.42 0 0 0 0 0.34 0 0 0 0 0.22 0 0 0 0.16 0'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23w)'/%3E%3C/svg%3E");
}
.sun {
  position: absolute;
  right: 16vw;
  top: 12vh;
  width: 16vh;
  height: 16vh;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(184, 50, 31, 0.85), rgba(184, 50, 31, 0.6) 60%, rgba(184, 50, 31, 0) 72%);
  filter: blur(0.5px);
}
.mountains {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 42vh;
}
.calligraphy {
  position: absolute;
  right: 4vw;
  top: 30vh;
  writing-mode: vertical-rl;
  font-family: "Yuji Syuku", "Shippori Mincho", serif;
  font-size: 15vh;
  line-height: 1;
  color: rgba(27, 27, 27, 0.08);
}
</style>
