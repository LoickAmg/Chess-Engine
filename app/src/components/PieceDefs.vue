<template>
  <!-- Dégradés partagés par toutes les pièces (définis une seule fois dans la page). -->
  <svg width="0" height="0" style="position: absolute" aria-hidden="true">
    <defs>
      <linearGradient id="pc-white" x1="0.15" y1="0" x2="0.85" y2="1">
        <stop offset="0" stop-color="#ffffff" />
        <stop offset="0.45" stop-color="#e4f7ff" />
        <stop offset="1" stop-color="#ffc6ec" />
      </linearGradient>
      <linearGradient id="pc-black" x1="0.15" y1="0" x2="0.85" y2="1">
        <stop offset="0" stop-color="#7a3fd0" />
        <stop offset="0.55" stop-color="#2d1060" />
        <stop offset="1" stop-color="#12052b" />
      </linearGradient>
      <radialGradient id="pc-gem-w" cx="0.35" cy="0.35" r="0.7">
        <stop offset="0" stop-color="#ffd6ef" />
        <stop offset="1" stop-color="#ff2f93" />
      </radialGradient>
      <radialGradient id="pc-gem-b" cx="0.35" cy="0.35" r="0.7">
        <stop offset="0" stop-color="#d9fbff" />
        <stop offset="1" stop-color="#16c5f0" />
      </radialGradient>
    </defs>
  </svg>
</template>
