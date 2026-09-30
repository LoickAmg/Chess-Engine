<script setup lang="ts">
// Fond animé : ciel violet profond, étoiles scintillantes, symboles de cartes géants et
// silhouettes de pièces qui dérivent lentement (un clin d'œil aux jeux de No Game No Life).
const SYMBOLS = [
  { ch: "♠", x: 6, y: 14, s: 160, d: 38, c: "cyan" },
  { ch: "♥", x: 84, y: 8, s: 120, d: 44, c: "pink" },
  { ch: "♦", x: 72, y: 70, s: 190, d: 52, c: "gold" },
  { ch: "♣", x: 18, y: 76, s: 130, d: 47, c: "violet" },
  { ch: "♛", x: 46, y: 36, s: 260, d: 60, c: "violet" },
  { ch: "♞", x: 92, y: 42, s: 110, d: 41, c: "cyan" },
  { ch: "♜", x: 34, y: 92, s: 90, d: 36, c: "pink" },
];
const STARS = Array.from({ length: 70 }, (_, i) => ({
  x: (i * 37.7) % 100,
  y: (i * 61.3) % 100,
  r: 1 + ((i * 7) % 3),
  d: 2 + ((i * 13) % 5),
}));
</script>

<template>
  <div class="cosmos" aria-hidden="true">
    <div class="nebula n1" />
    <div class="nebula n2" />
    <div class="nebula n3" />
    <span
      v-for="(s, i) in STARS"
      :key="`s${i}`"
      class="dot"
      :style="{ left: `${s.x}%`, top: `${s.y}%`, width: `${s.r}px`, height: `${s.r}px`, animationDuration: `${s.d}s` }"
    />
    <span
      v-for="(s, i) in SYMBOLS"
      :key="`c${i}`"
      class="symbol"
      :class="s.c"
      :style="{ left: `${s.x}%`, top: `${s.y}%`, fontSize: `${s.s}px`, animationDuration: `${s.d}s` }"
    >{{ s.ch }}</span>
    <div class="grid-floor" />
  </div>
</template>

<style scoped>
.cosmos {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  background: radial-gradient(ellipse 120% 90% at 50% 0%, #2a0d63 0%, #140536 45%, #07011a 100%);
  pointer-events: none;
}
.nebula {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.5;
  animation: drift 40s ease-in-out infinite alternate;
}
.n1 {
  width: 50vw;
  height: 50vw;
  left: -12vw;
  top: -18vw;
  background: #ff4fa3;
  opacity: 0.22;
}
.n2 {
  width: 45vw;
  height: 45vw;
  right: -14vw;
  top: 20vh;
  background: #32e0ff;
  opacity: 0.16;
  animation-duration: 55s;
}
.n3 {
  width: 40vw;
  height: 40vw;
  left: 30vw;
  bottom: -25vw;
  background: #9b6bff;
  opacity: 0.3;
  animation-duration: 48s;
}
@keyframes drift {
  to {
    transform: translate(6vw, 4vh) scale(1.1);
  }
}
.dot {
  position: absolute;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 0 6px #fff;
  animation: twinkle ease-in-out infinite;
}
@keyframes twinkle {
  50% {
    opacity: 0.2;
  }
}
.symbol {
  position: absolute;
  line-height: 1;
  font-family: "Segoe UI Symbol", sans-serif;
  opacity: 0.07;
  transform: translate(-50%, -50%);
  animation: sway ease-in-out infinite alternate;
}
.symbol.cyan {
  color: #32e0ff;
}
.symbol.pink {
  color: #ff4fa3;
}
.symbol.gold {
  color: #ffd166;
}
.symbol.violet {
  color: #b58cff;
}
@keyframes sway {
  from {
    transform: translate(-50%, -50%) rotate(-8deg);
  }
  to {
    transform: translate(-50%, -58%) rotate(8deg);
  }
}
.grid-floor {
  position: absolute;
  left: -20%;
  right: -20%;
  bottom: -10%;
  height: 45%;
  background-image:
    linear-gradient(rgba(155, 107, 255, 0.18) 1px, transparent 1px),
    linear-gradient(90deg, rgba(155, 107, 255, 0.18) 1px, transparent 1px);
  background-size: 60px 60px;
  transform: perspective(500px) rotateX(62deg);
  transform-origin: bottom;
  mask-image: linear-gradient(to top, #000 0%, transparent 85%);
  animation: scroll-grid 6s linear infinite;
}
@keyframes scroll-grid {
  to {
    background-position: 0 60px;
  }
}
</style>
