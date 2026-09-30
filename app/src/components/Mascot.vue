<script setup lang="ts">
import { computed } from "vue";
import { useProgressStore } from "@/stores/progress";
import ChessPiece from "./ChessPiece.vue";

// Academy, le professeur : un emblème qui prend la forme du thème choisi.
//  Classique : médaillon noir, filet d'or et roi Staunton ivoire.
//  Persona 5 : étoile rouge éclatée, roi blanc au trait épais.
//  Persona 3 : blason bleu nuit, roi de cristal.
//  Encre : sceau vermillon « 師 » (maître).
const props = withDefaults(defineProps<{ mood?: "happy" | "think" | "wow" | "sad"; size?: number }>(), {
  mood: "happy",
  size: 96,
});
const progress = useProgressStore();
const theme = computed(() => progress.settings.theme);
</script>

<template>
  <div class="prof" :class="[`t-${theme}`, `m-${props.mood}`]" :style="{ width: `${size}px`, height: `${size}px` }" aria-hidden="true">
    <template v-if="theme === 'sumi'">
      <div class="seal"><span :style="{ fontSize: `${size * 0.46}px` }">師</span></div>
    </template>
    <template v-else-if="theme === 'persona5'">
      <svg class="burst" viewBox="0 0 100 100">
        <polygon
          points="50,2 60,24 84,10 76,36 98,44 76,56 90,82 62,72 54,98 44,74 18,90 26,62 2,54 24,42 12,16 38,26"
          fill="#e0001b"
          stroke="#0b0b0b"
          stroke-width="3"
        />
      </svg>
      <div class="piece"><ChessPiece kind="k" color="w" set="persona" /></div>
    </template>
    <template v-else>
      <div class="medal">
        <div class="piece">
          <ChessPiece kind="k" :color="theme === 'classic' ? 'w' : 'w'" :set="theme === 'persona3' ? 'crystal' : 'staunton'" />
        </div>
      </div>
    </template>
    <span v-if="props.mood === 'think'" class="dots"><i /><i /><i /></span>
  </div>
</template>

<style scoped>
.prof {
  position: relative;
  flex: none;
  display: grid;
  place-items: center;
  transition: transform 0.4s var(--spring);
}
.m-wow {
  animation: hop 0.6s var(--spring);
}
.m-sad {
  animation: shake 0.45s;
}
@keyframes hop {
  40% {
    transform: translateY(-8px) scale(1.04);
  }
}
@keyframes shake {
  25% {
    transform: translateX(-4px) rotate(-3deg);
  }
  75% {
    transform: translateX(4px) rotate(3deg);
  }
}

/* Classique et Persona 3 : médaillon */
.medal {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, #3a3129, #14110e 70%);
  box-shadow:
    inset 0 0 0 2px #c9a96a,
    inset 0 0 0 5px #14110e,
    inset 0 0 0 6px rgba(201, 169, 106, 0.6),
    0 10px 24px rgba(40, 25, 10, 0.3);
}
.t-persona3 .medal {
  border-radius: 12% 50% 12% 50%;
  background: radial-gradient(circle at 35% 30%, #1f4fbf, #061640 72%);
  box-shadow: inset 0 0 0 2px #3fe0ff, 0 0 22px rgba(63, 224, 255, 0.45);
}
.piece {
  width: 62%;
  height: 62%;
  margin-top: -4%;
}

/* Persona 5 : étoile éclatée */
.burst {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  animation: spin 18s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.t-persona5 .piece {
  position: relative;
  width: 56%;
  height: 56%;
}

/* Encre : sceau (hanko) */
.seal {
  width: 82%;
  height: 82%;
  display: grid;
  place-items: center;
  border-radius: 8%;
  background: #b8321f;
  box-shadow: inset 0 0 0 3px rgba(255, 240, 225, 0.35), 0 8px 18px rgba(120, 30, 15, 0.25);
  transform: rotate(-5deg);
  -webkit-mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='100' height='100'%3E%3Cfilter id='r'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2'/%3E%3CfeColorMatrix values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 -2.2 2.1'/%3E%3C/filter%3E%3Crect width='100' height='100' filter='url(%23r)'/%3E%3C/svg%3E");
  mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='100' height='100'%3E%3Cfilter id='r'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2'/%3E%3CfeColorMatrix values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 -2.2 2.1'/%3E%3C/filter%3E%3Crect width='100' height='100' filter='url(%23r)'/%3E%3C/svg%3E");
  mask-size: 100% 100%;
}
.seal span {
  font-family: "Yuji Syuku", "Shippori Mincho", serif;
  line-height: 1;
  color: #f7eee0;
}

/* Réflexion : trois points */
.dots {
  position: absolute;
  right: -6px;
  top: -2px;
  display: flex;
  gap: 3px;
  padding: 4px 6px;
  border-radius: 10px;
  background: var(--bubble-bg);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}
.dots i {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--accent);
  animation: bounce 1s ease-in-out infinite;
}
.dots i:nth-child(2) {
  animation-delay: 0.15s;
}
.dots i:nth-child(3) {
  animation-delay: 0.3s;
}
@keyframes bounce {
  50% {
    transform: translateY(-4px);
  }
}
</style>
