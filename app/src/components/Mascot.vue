<script setup lang="ts">
// Lumi, la professeure : une petite dame de cristal (pièce originale) avec un visage,
// qui flotte, cligne des yeux et change d'expression selon la situation.
withDefaults(defineProps<{ mood?: "happy" | "think" | "wow" | "sad"; size?: number }>(), {
  mood: "happy",
  size: 96,
});
</script>

<template>
  <svg class="mascot" :class="mood" :width="size" :height="size" viewBox="0 0 100 100" aria-hidden="true">
    <defs>
      <radialGradient id="m-aura" cx="0.5" cy="0.5" r="0.5">
        <stop offset="0" stop-color="#ff7ac2" stop-opacity="0.45" />
        <stop offset="1" stop-color="#ff7ac2" stop-opacity="0" />
      </radialGradient>
    </defs>
    <circle cx="50" cy="52" r="48" fill="url(#m-aura)" class="aura" />
    <g class="body" stroke="#3b1a6e" stroke-width="2.6" stroke-linejoin="round" fill="url(#pc-white)">
      <!-- couronne -->
      <path d="M27 30 L34 50 H66 L73 30 L61 41 L56 20 L50 38 L44 20 L39 41 Z" />
      <circle cx="27" cy="27" r="4" />
      <circle cx="44" cy="17" r="4" />
      <circle cx="56" cy="17" r="4" />
      <circle cx="73" cy="27" r="4" />
      <path d="M50 44 C46 41 44 39 44 36.8 C44 34.9 45.8 34 47.4 34 C48.6 34 49.6 34.7 50 35.7 C50.4 34.7 51.4 34 52.6 34 C54.2 34 56 34.9 56 36.8 C56 39 54 41 50 44 Z" fill="url(#pc-gem-w)" stroke-width="1.4" />
      <!-- visage -->
      <path d="M30 50 H70 C72 62 68 76 50 80 C32 76 28 62 30 50 Z" />
      <!-- socle -->
      <path d="M26 88 Q50 82 74 88 L77 95 Q50 100 23 95 Z" />
      <path d="M40 80 L42 86 H58 L60 80" />
    </g>
    <g class="face">
      <template v-if="mood === 'think'">
        <path d="M38 62 q4 -3 8 0" class="line" />
        <path d="M54 62 q4 -3 8 0" class="line" />
        <path d="M45 71 h10" class="line" />
      </template>
      <template v-else-if="mood === 'sad'">
        <ellipse cx="42" cy="62" rx="2.6" ry="3.2" class="eye" />
        <ellipse cx="58" cy="62" rx="2.6" ry="3.2" class="eye" />
        <path d="M44 73 q6 -4 12 0" class="line" />
      </template>
      <template v-else>
        <g class="eyes">
          <ellipse cx="42" cy="62" rx="3" :ry="mood === 'wow' ? 4.2 : 3.6" class="eye" />
          <ellipse cx="58" cy="62" rx="3" :ry="mood === 'wow' ? 4.2 : 3.6" class="eye" />
          <circle cx="43" cy="60.6" r="1" fill="#fff" />
          <circle cx="59" cy="60.6" r="1" fill="#fff" />
        </g>
        <ellipse v-if="mood === 'wow'" cx="50" cy="72" rx="3" ry="3.4" class="eye" />
        <path v-else d="M45 70 q5 5 10 0" class="line" />
      </template>
      <ellipse cx="36" cy="68" rx="3.4" ry="2" fill="#ff8fc7" opacity="0.7" />
      <ellipse cx="64" cy="68" rx="3.4" ry="2" fill="#ff8fc7" opacity="0.7" />
    </g>
  </svg>
</template>

<style scoped>
.mascot {
  overflow: visible;
  animation: float 3.6s ease-in-out infinite;
  filter: drop-shadow(0 8px 10px rgba(10, 0, 30, 0.5));
}
.aura {
  animation: breathe 3.6s ease-in-out infinite;
  transform-origin: 50% 52%;
}
@keyframes breathe {
  50% {
    transform: scale(1.1);
    opacity: 0.7;
  }
}
.eye {
  fill: #3b1a6e;
}
.eyes {
  transform-origin: 50% 62%;
  animation: blink 4.5s infinite;
}
@keyframes blink {
  0%,
  46%,
  50%,
  100% {
    transform: scaleY(1);
  }
  48% {
    transform: scaleY(0.1);
  }
}
.line {
  fill: none;
  stroke: #3b1a6e;
  stroke-width: 2.2;
  stroke-linecap: round;
}
.wow {
  animation: float 3.6s ease-in-out infinite, hop 0.5s var(--spring);
}
@keyframes hop {
  40% {
    transform: translateY(-12px) scale(1.05);
  }
}
</style>
