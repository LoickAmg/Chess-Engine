<script setup lang="ts">
// Icônes maison (grille 24 × 24, trait arrondi).
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 20 });

const PATHS: Record<string, string> = {
  home: "M4 10.5 12 4l8 6.5V19a1 1 0 0 1-1 1h-4.5v-5.5h-5V20H5a1 1 0 0 1-1-1Z",
  book: "M5 4.5h9.5a3 3 0 0 1 3 3V20H8a3 3 0 0 1-3-3ZM5 17a3 3 0 0 1 3-3h9.5M9 8.5h5",
  swords: "M4 4l9 9M4 4h4M4 4v4M20 4l-9 9M20 4h-4M20 4v4M7 14l3 3M17 14l-3 3M5.5 18.5 8 16M18.5 18.5 16 16",
  target: "M12 20.5a8.5 8.5 0 1 1 0-17 8.5 8.5 0 0 1 0 17ZM12 16a4 4 0 1 1 0-8 4 4 0 0 1 0 8ZM12 12h.01",
  puzzle:
    "M9 4.5h2.2a1.8 1.8 0 1 1 3.6 0H17a1.5 1.5 0 0 1 1.5 1.5v2.2a1.8 1.8 0 1 1 0 3.6V14a1.5 1.5 0 0 1-1.5 1.5h-2.2a1.8 1.8 0 1 0-3.6 0H9A1.5 1.5 0 0 1 7.5 14v-2.2a1.8 1.8 0 1 0 0-3.6V6A1.5 1.5 0 0 1 9 4.5Z",
  library: "M5 4.5v15M9 4.5v15M13.5 5l4.5 14.2M3.5 19.5h17",
  star: "M12 3.5l2.6 5.4 5.9.8-4.3 4.1 1 5.9L12 17l-5.2 2.7 1-5.9-4.3-4.1 5.9-.8Z",
  check: "M5 12.5l4.2 4.2L19 7",
  lock: "M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z",
  play: "M8 5.5v13l10.5-6.5Z",
  back: "M15 5.5 8.5 12l6.5 6.5",
  next: "M9 5.5l6.5 6.5L9 18.5",
  undo: "M9 7 4.5 11.5 9 16M4.5 11.5H15a4.5 4.5 0 0 1 0 9h-2",
  flag: "M6 20.5V4M6 4.5h11l-2.5 4L17 12.5H6",
  bulb: "M9 17.5h6M10 20.5h4M12 3.5a6 6 0 0 1 3.6 10.8c-.7.5-1.1 1.2-1.1 2v.2h-5v-.2c0-.8-.4-1.5-1.1-2A6 6 0 0 1 12 3.5Z",
  flip: "M7 4v16M7 20l-3-3M7 20l3-3M17 20V4M17 4l-3 3M17 4l3 3",
  sound: "M4 9.5h3.5L12 5.5v13l-4.5-4H4zM15.5 9a4.2 4.2 0 0 1 0 6M18 6.5a7.8 7.8 0 0 1 0 11",
  mute: "M4 9.5h3.5L12 5.5v13l-4.5-4H4zM16 9.5l5 5M21 9.5l-5 5",
  refresh: "M19.5 12a7.5 7.5 0 1 1-2.2-5.3M19.5 4.5v4h-4",
  timer: "M12 20.5a7.5 7.5 0 1 1 0-15 7.5 7.5 0 0 1 0 15ZM12 9v4l2.5 1.5M9.5 2.5h5",
  trophy: "M8 4.5h8v5a4 4 0 0 1-8 0ZM8 6.5H5v1.5a3 3 0 0 0 3 3M16 6.5h3v1.5a3 3 0 0 1-3 3M12 13.5v3M8.5 20h7M9.5 16.5h5V20h-5z",
  close: "M6 6l12 12M18 6 6 18",
  search: "M10.5 17a6.5 6.5 0 1 1 0-13 6.5 6.5 0 0 1 0 13ZM15.3 15.3 20 20",
  board: "M4.5 4.5h15v15h-15zM4.5 9.5h15M4.5 14.5h15M9.5 4.5v15M14.5 4.5v15",
  setup: "M4 19.5h16M6.5 19.5v-4h3v4M14.5 19.5v-6h3v6M10.5 19.5V9.5h3v10",
  gem: "M7 4.5h10l3.5 5L12 20 3.5 9.5ZM3.5 9.5h17M9.5 4.5 8 9.5l4 10.5 4-10.5-1.5-5",
  alert: "M12 4 21 19.5H3ZM12 10v4.5M12 17h.01",
  crown: "M4 8l4 4 4-7 4 7 4-4-1.5 10.5h-13ZM6 20.5h12",
  pause: "M12 20.5a8.5 8.5 0 1 1 0-17 8.5 8.5 0 0 1 0 17ZM10 9v6M14 9v6",
  scale: "M12 4v16M7 20h10M5 7h14M5 7l-2.5 6a2.5 2.5 0 0 0 5 0ZM19 7l-2.5 6a2.5 2.5 0 0 0 5 0Z",
  castle: "M5 20.5v-12h3v2.5h2.5V8.5h3V11H16V8.5h3v12ZM10 20.5v-4a2 2 0 0 1 4 0v4",
  swap: "M7 4.5v15M7 19.5l-3-3M17 19.5v-15M17 4.5l3 3",
  pen: "M15 4.5 19.5 9 9 19.5H4.5V15ZM13 6.5l4.5 4.5",
  rocket: "M12 3.5c3.5 2 5 5.5 4.5 10l-2 2.5h-5l-2-2.5c-.5-4.5 1-8 4.5-10ZM12 10.5h.01M9.5 16l-2.5 4 3.5-1.5M14.5 16l2.5 4-3.5-1.5",
  shield: "M12 3.5l7.5 3v5.5c0 4.5-3.2 7.5-7.5 8.5-4.3-1-7.5-4-7.5-8.5V6.5ZM8.5 12l2.5 2.5 4.5-4.5",
  fork: "M7 4v6a5 5 0 0 0 10 0V4M12 4v16M7 4v3M17 4v3",
  pin: "M9 4.5h6l-1 5 3 3H7l3-3ZM12 12.5v8",
  skewer: "M3.5 12h17M16.5 8l4 4-4 4M8 9.5a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5Z",
  eye: "M2.5 12s3.5-6.5 9.5-6.5S21.5 12 21.5 12s-3.5 6.5-9.5 6.5S2.5 12 2.5 12ZM12 15a3 3 0 1 1 0-6 3 3 0 0 1 0 6Z",
  wall: "M3.5 6.5h17v11h-17zM3.5 12h17M9 6.5V12M15 6.5V12M6 12v5.5M12 12v5.5M18 12v5.5",
  stairs: "M4 19.5h4.5V15H13v-4.5h4.5V6h3",
  kings: "M7 20.5v-6M17 20.5v-6M4.5 14.5h5M14.5 14.5h5M7 8.5v-4M5.5 6h3M17 8.5v-4M15.5 6h3M5 14.5a2 3 0 0 1 4 0M15 14.5a2 3 0 0 1 4 0",
  square: "M5 5h14v14H5zM5 5l14 14",
  sparkle: "M12 3.5 13.8 10.2 20.5 12l-6.7 1.8L12 20.5l-1.8-6.7L3.5 12l6.7-1.8Z",
};
const FILLED = new Set(["play", "star", "sparkle"]);
</script>

<template>
  <svg
    class="icon"
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    :fill="FILLED.has(props.name) ? 'currentColor' : 'none'"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path :d="PATHS[props.name] ?? PATHS.sparkle" />
  </svg>
</template>

<style scoped>
.icon {
  flex: none;
  display: block;
}
</style>
