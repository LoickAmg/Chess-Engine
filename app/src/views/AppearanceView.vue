<script setup lang="ts">
import type { PieceKind } from "@/lib/chess";
import { BOARDS, PIECE_SETS, THEMES, type BoardId, type PieceSetId, type ThemeId } from "@/lib/themes";
import { useProgressStore } from "@/stores/progress";
import ChessBoard from "@/components/ChessBoard.vue";
import ChessPiece from "@/components/ChessPiece.vue";
import Mascot from "@/components/Mascot.vue";

// Apparence : thème, échiquier et pièces se choisissent séparément ; choisir un thème
// applique aussi son plateau et ses pièces, qu'on peut ensuite changer librement.
const progress = useProgressStore();

const PREVIEW = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 1 5";
const KINDS: PieceKind[] = ["k", "q", "r", "b", "n", "p"];

function pickTheme(id: ThemeId) {
  const t = THEMES.find((x) => x.id === id)!;
  progress.setSetting("theme", id);
  progress.setSetting("board", t.board);
  progress.setSetting("pieces", t.pieces);
}
const pickBoard = (id: BoardId) => progress.setSetting("board", id);
const pickPieces = (id: PieceSetId) => progress.setSetting("pieces", id);
</script>

<template>
  <div class="page">
    <div class="cols">
      <div class="options">
        <p class="eyebrow">Personnalisation</p>
        <h1>Apparence</h1>
        <p class="muted intro">Choisis un thème : il règle l'interface, l'échiquier et les pièces. Tu peux ensuite changer l'échiquier ou les pièces indépendamment.</p>

        <h2>Thème</h2>
        <div class="themes">
          <button
            v-for="t in THEMES"
            :key="t.id"
            type="button"
            class="theme"
            :class="[`th-${t.id}`, { on: progress.settings.theme === t.id }]"
            @click="pickTheme(t.id)"
          >
            <span class="swatches">
              <i v-for="c in t.swatch" :key="c" :style="{ background: c }" />
            </span>
            <strong>{{ t.name }}</strong>
            <small>{{ t.tagline }}</small>
          </button>
        </div>

        <h2>Échiquier</h2>
        <div class="boards">
          <button
            v-for="b in BOARDS"
            :key="b.id"
            type="button"
            class="board-card"
            :class="{ on: progress.settings.board === b.id }"
            @click="pickBoard(b.id)"
          >
            <div class="mini"><ChessBoard fen="8/8/8/8/8/8/8/8 w - - 0 1" :board-style="b.id" :show-coords="false" /></div>
            <span>{{ b.name }}</span>
          </button>
        </div>

        <h2>Pièces</h2>
        <div class="sets">
          <button
            v-for="s in PIECE_SETS"
            :key="s.id"
            type="button"
            class="set"
            :class="{ on: progress.settings.pieces === s.id }"
            @click="pickPieces(s.id)"
          >
            <span class="row">
              <span v-for="k in KINDS" :key="`w${k}`" class="pc"><ChessPiece :kind="k" color="w" :set="s.id" /></span>
            </span>
            <span class="row dark">
              <span v-for="k in KINDS" :key="`b${k}`" class="pc"><ChessPiece :kind="k" color="b" :set="s.id" /></span>
            </span>
            <strong>{{ s.name }}</strong>
            <small>{{ s.description }}</small>
          </button>
        </div>
      </div>

      <aside class="preview glass">
        <div class="prof-row">
          <Mascot :size="56" />
          <p class="bubble">Voilà ton échiquier. Il sera le même dans les leçons, les puzzles et les parties.</p>
        </div>
        <ChessBoard :fen="PREVIEW" :last-move="['g1', 'f3']" />
      </aside>
    </div>
  </div>
</template>

<style scoped>
.page {
  height: 100%;
  overflow-y: auto;
  padding: 36px 44px 48px;
}
.cols {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(320px, 440px);
  gap: 36px;
  align-items: start;
}
h1 {
  font-size: 46px;
  margin: 6px 0 8px;
}
h2 {
  margin: 28px 0 12px;
  font-size: 22px;
}
.intro {
  max-width: 620px;
  margin: 0;
}
button {
  text-align: left;
  border: 0;
  background: none;
}

/* Thèmes */
.themes {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}
.theme {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: inset 0 0 0 1px var(--line);
  transition: transform 0.2s var(--ease), box-shadow 0.2s;
}
.theme:hover {
  transform: translateY(-2px);
}
.theme.on,
.board-card.on,
.set.on {
  box-shadow: inset 0 0 0 2px var(--accent), 0 8px 22px rgba(0, 0, 0, 0.12);
}
.swatches {
  display: flex;
  gap: 5px;
  margin-bottom: 4px;
}
.swatches i {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.15);
}
.th-persona5 strong {
  font-family: "Anton", sans-serif;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.th-persona3 strong {
  font-family: "Barlow Condensed", sans-serif;
  font-style: italic;
  text-transform: uppercase;
}
.th-sumi strong {
  font-family: "Yuji Syuku", serif;
  font-weight: 400;
}
.th-classic strong {
  font-family: "Cormorant Garamond", serif;
  font-size: 22px;
}
.theme strong {
  font-size: 20px;
}
.theme small,
.set small {
  color: var(--ink-2);
  font-size: 12.5px;
  line-height: 1.45;
}

/* Échiquiers */
.boards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 12px;
}
.board-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: inset 0 0 0 1px var(--line);
  font-weight: 600;
  font-size: 13px;
}
.mini {
  pointer-events: none;
}
.mini :deep(.frame) {
  --frame-pad: 6px;
}

/* Pièces */
.sets {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}
.set {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px;
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: inset 0 0 0 1px var(--line);
}
.row {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  padding: 4px 6px;
  border-radius: 4px;
  background: #e6cfa6;
}
.row.dark {
  background: #2b211a;
  margin-bottom: 8px;
}
.pc {
  aspect-ratio: 1;
}
.set strong {
  font-size: 16px;
}

/* Aperçu */
.preview {
  position: sticky;
  top: 0;
  padding: 18px;
}
.prof-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.prof-row .bubble {
  font-size: 13.5px;
  padding: 10px 14px;
}
@media (max-width: 1150px) {
  .cols {
    grid-template-columns: 1fr;
  }
  .preview {
    position: static;
    max-width: 480px;
  }
}
</style>
