<template>
  <!-- Matières des pièces (dégradés et filtres), définies une seule fois dans la page.
       Les dégradés horizontaux donnent le volume d'une pièce tournée : bord sombre,
       reflet de lumière aux deux tiers gauches, retour à l'ombre. -->
  <svg width="0" height="0" style="position: absolute" aria-hidden="true" focusable="false">
    <defs>
      <!-- Staunton : buis et ébène -->
      <linearGradient id="pc-staunton-w" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#a88257" />
        <stop offset="0.22" stop-color="#e3caa0" />
        <stop offset="0.4" stop-color="#fbf0db" />
        <stop offset="0.52" stop-color="#f2e1c2" />
        <stop offset="0.8" stop-color="#cfae80" />
        <stop offset="1" stop-color="#94703f" />
      </linearGradient>
      <linearGradient id="pc-staunton-b" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#050403" />
        <stop offset="0.24" stop-color="#2c211a" />
        <stop offset="0.4" stop-color="#5b4739" />
        <stop offset="0.52" stop-color="#3b2e25" />
        <stop offset="0.8" stop-color="#15100c" />
        <stop offset="1" stop-color="#030202" />
      </linearGradient>
      <linearGradient id="pc-gloss" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#fff" stop-opacity="0.38" />
        <stop offset="0.45" stop-color="#fff" stop-opacity="0.05" />
        <stop offset="1" stop-color="#fff" stop-opacity="0" />
      </linearGradient>

      <!-- Cristal : verre taillé clair / verre fumé -->
      <linearGradient id="pc-crystal-w" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#9dbad3" stop-opacity="0.7" />
        <stop offset="0.3" stop-color="#e8f6ff" stop-opacity="0.75" />
        <stop offset="0.45" stop-color="#ffffff" stop-opacity="0.95" />
        <stop offset="0.62" stop-color="#cfe3f2" stop-opacity="0.6" />
        <stop offset="1" stop-color="#7fa3c2" stop-opacity="0.75" />
      </linearGradient>
      <linearGradient id="pc-crystal-b" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#0c0e18" stop-opacity="0.85" />
        <stop offset="0.3" stop-color="#3a3f5c" stop-opacity="0.7" />
        <stop offset="0.45" stop-color="#8d95bf" stop-opacity="0.75" />
        <stop offset="0.62" stop-color="#2b2f48" stop-opacity="0.7" />
        <stop offset="1" stop-color="#07080f" stop-opacity="0.9" />
      </linearGradient>

      <!-- Holographique : verre irisé -->
      <linearGradient id="pc-holo-w" x1="0" y1="0" x2="0.4" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.9" />
        <stop offset="0.3" stop-color="#7ff3ff" stop-opacity="0.6" />
        <stop offset="0.6" stop-color="#ff7ad9" stop-opacity="0.55" />
        <stop offset="1" stop-color="#ffe36b" stop-opacity="0.65" />
      </linearGradient>
      <linearGradient id="pc-holo-b" x1="0" y1="0" x2="0.4" y2="1">
        <stop offset="0" stop-color="#1b0b3a" stop-opacity="0.9" />
        <stop offset="0.35" stop-color="#6a2cff" stop-opacity="0.75" />
        <stop offset="0.7" stop-color="#0bb8ff" stop-opacity="0.7" />
        <stop offset="1" stop-color="#ff2d8a" stop-opacity="0.75" />
      </linearGradient>

      <!-- Encre : lavis -->
      <radialGradient id="pc-ink-b" cx="0.35" cy="0.3" r="0.9">
        <stop offset="0" stop-color="#4a4640" />
        <stop offset="0.55" stop-color="#1c1b19" />
        <stop offset="1" stop-color="#050505" />
      </radialGradient>
      <radialGradient id="pc-ink-w" cx="0.4" cy="0.3" r="0.9">
        <stop offset="0" stop-color="#fbf6ea" />
        <stop offset="1" stop-color="#e6dac1" />
      </radialGradient>
      <!-- Trait de pinceau irrégulier -->
      <filter id="pc-brush" x="-10%" y="-10%" width="120%" height="120%">
        <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="4" result="noise" />
        <feDisplacementMap in="SourceGraphic" in2="noise" scale="2.4" xChannelSelector="R" yChannelSelector="G" />
      </filter>

      <!-- Lueur des pièces de verre -->
      <filter id="pc-glow" x="-30%" y="-30%" width="160%" height="160%">
        <feGaussianBlur stdDeviation="1.6" result="b" />
        <feMerge>
          <feMergeNode in="b" />
          <feMergeNode in="SourceGraphic" />
        </feMerge>
      </filter>
    </defs>
  </svg>
</template>
