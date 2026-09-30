<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import type { QualityId } from "@/lib/api";
import { parseFen, PIECE_VALUES, type Arrow, type Color, type PieceKind } from "@/lib/chess";
import { rich } from "@/lib/content";
import { LEVELS, useGameStore } from "@/stores/game";
import { useProgressStore } from "@/stores/progress";
import ChessBoard from "@/components/ChessBoard.vue";
import ChessPiece from "@/components/ChessPiece.vue";
import Icon from "@/components/Icon.vue";
import Mascot from "@/components/Mascot.vue";

const game = useGameStore();
const progress = useProgressStore();

// ------------------------------------------------------------------ préparation
const pickLevel = ref(progress.settings.level);
const pickColor = ref<Color | "random">("w");
const pickCoach = ref(progress.settings.coachTalk);
const flipped = ref(false);
const showBest = ref(false);

function start() {
  progress.setSetting("coachTalk", pickCoach.value);
  flipped.value = false;
  void game.start(pickLevel.value, pickColor.value, pickCoach.value);
}

const orientation = computed<Color>(() => {
  const base = game.you;
  return flipped.value ? (base === "w" ? "b" : "w") : base;
});
const levelInfo = computed(() => LEVELS[game.level - 1]);

// ------------------------------------------------------------------ plateau
const arrows = computed<Arrow[]>(() => {
  const out: Arrow[] = [];
  if (game.hint) out.push({ from: game.hint.uci.slice(0, 2), to: game.hint.uci.slice(2, 4), color: "gold" });
  const r = game.lastReview;
  if (showBest.value && r?.best_uci) out.push({ from: r.best_uci.slice(0, 2), to: r.best_uci.slice(2, 4), color: "green" });
  return out;
});
watch(
  () => game.plies.length,
  () => (showBest.value = false),
);

// Matériel capturé et avantage
const material = computed(() => {
  const start: Record<PieceKind, number> = { p: 8, n: 2, b: 2, r: 2, q: 1, k: 1 };
  const board = parseFen(game.fen).board;
  const left: Record<Color, Record<PieceKind, number>> = {
    w: { p: 0, n: 0, b: 0, r: 0, q: 0, k: 0 },
    b: { p: 0, n: 0, b: 0, r: 0, q: 0, k: 0 },
  };
  for (const p of board.values()) left[p.color][p.kind]++;
  const lost = (c: Color) =>
    (["q", "r", "b", "n", "p"] as PieceKind[]).flatMap((k) => Array(Math.max(0, start[k] - left[c][k])).fill(k) as PieceKind[]);
  const score = (c: Color) => (Object.keys(left[c]) as PieceKind[]).reduce((s, k) => s + left[c][k] * PIECE_VALUES[k], 0);
  return { lostW: lost("w"), lostB: lost("b"), diff: score("w") - score("b") };
});
// Pièces prises PAR un camp = pièces perdues par l'autre camp.
const takenBy = (c: Color) => (c === "w" ? material.value.lostB : material.value.lostW);
const advantage = (c: Color) => (c === "w" ? material.value.diff : -material.value.diff);

// Barre d'évaluation (point de vue des Blancs, en centipions)
const evalPct = computed(() => {
  const cp = Math.max(-2000, Math.min(2000, game.evalWhite));
  return 50 + 50 * Math.tanh(cp / 500);
});
const evalLabel = computed(() => {
  const cp = game.evalWhite;
  if (Math.abs(cp) > 20000) return cp > 0 ? "Mat" : "-Mat";
  return (cp > 0 ? "+" : "") + (cp / 100).toFixed(1).replace(".", ",");
});

// ------------------------------------------------------------------ commentaires
const QUALITY: Record<QualityId, { tone: string; icon: string }> = {
  best: { tone: "gold", icon: "star" },
  excellent: { tone: "green", icon: "sparkle" },
  good: { tone: "cyan", icon: "check" },
  inaccuracy: { tone: "orange", icon: "alert" },
  mistake: { tone: "pink", icon: "alert" },
  blunder: { tone: "red", icon: "close" },
};
const mood = computed(() => {
  if (game.outcome) return game.outcome.result === "win" ? "wow" : game.outcome.result === "loss" ? "happy" : "think";
  if (game.thinking) return "think";
  const q = game.lastReview?.quality;
  if (q === "best" || q === "excellent") return "wow";
  if (q === "mistake" || q === "blunder") return "sad";
  return "happy";
});
const coachText = computed(() => {
  if (!game.started) return "";
  if (game.thinking) return "Hmm… je réfléchis à mon coup.";
  if (game.hint) return `À ta place, je regarderais **${game.hint.san}** (flèche dorée).`;
  if (!game.plies.length) return game.you === "w" ? "À toi de commencer ! Pense au centre." : "J'ai commencé, à toi de répondre !";
  if (!game.coach) return "Mode sans commentaires : bonne partie !";
  if (game.reviewing && !game.lastReview) return "J'analyse ton coup…";
  return "";
});

// Liste des coups par paires (1. e4 e5)
const rows = computed(() => {
  const out: { n: number; w?: (typeof game.plies)[number]; b?: (typeof game.plies)[number] }[] = [];
  game.plies.forEach((p, idx) => {
    const n = Math.floor(idx / 2) + 1;
    if (idx % 2 === 0) out.push({ n, w: p });
    else out[out.length - 1].b = p;
  });
  return out;
});
const movesBox = ref<HTMLElement | null>(null);
watch(
  () => game.plies.length,
  () => nextTick(() => movesBox.value?.scrollTo({ top: movesBox.value.scrollHeight, behavior: "smooth" })),
);

const resultTitle = computed(() => {
  const o = game.outcome;
  if (!o) return "";
  return o.result === "win" ? "Victoire !" : o.result === "loss" ? "Défaite" : "Partie nulle";
});
const resultText = computed(() => {
  const o = game.outcome;
  if (!o) return "";
  if (o.result === "win") return "Bravo, tu m'as battue ! Essaie le niveau suivant quand tu te sens prêt(e).";
  if (o.result === "loss") return "Ce n'est pas grave : chaque défaite t'apprend quelque chose. Regarde tes erreurs dans la liste des coups !";
  return "Match nul. Une partie bien défendue, c'est déjà une belle performance.";
});
</script>

<template>
  <div class="play">
    <!-- Choix de la partie -->
    <div v-if="!game.started" class="setup">
      <div class="setup-card glass">
        <div class="setup-head">
          <Mascot :size="96" mood="happy" />
          <div>
            <p class="eyebrow">Partie</p>
            <h1>Jouer contre Lumi</h1>
            <p class="muted">Je joue contre toi et je commente chacun de tes coups, comme un vrai professeur.</p>
          </div>
        </div>

        <h3>Mon niveau</h3>
        <div class="levels">
          <button
            v-for="l in LEVELS"
            :key="l.level"
            type="button"
            class="level"
            :class="{ on: pickLevel === l.level }"
            @click="pickLevel = l.level"
          >
            <span class="stars"><Icon v-for="i in l.level" :key="i" name="star" :size="12" /></span>
            <strong>{{ l.name }}</strong>
            <small>{{ l.desc }}</small>
          </button>
        </div>

        <h3>Ta couleur</h3>
        <div class="colors">
          <button type="button" class="color" :class="{ on: pickColor === 'w' }" @click="pickColor = 'w'">
            <span class="swatch"><ChessPiece kind="k" color="w" /></span> Blancs
          </button>
          <button type="button" class="color" :class="{ on: pickColor === 'b' }" @click="pickColor = 'b'">
            <span class="swatch"><ChessPiece kind="k" color="b" /></span> Noirs
          </button>
          <button type="button" class="color" :class="{ on: pickColor === 'random' }" @click="pickColor = 'random'">
            <span class="swatch dice">?</span> Au hasard
          </button>
        </div>

        <label class="toggle">
          <input v-model="pickCoach" type="checkbox" />
          <span class="switch" />
          <span>
            <strong>Commentaires du professeur</strong><br />
            <small class="muted">J'analyse chaque coup : qualité, pièces en prise, meilleur coup, principes.</small>
          </span>
        </label>

        <button type="button" class="btn btn-primary big" @click="start"><Icon name="play" :size="16" /> Commencer la partie</button>
      </div>
    </div>

    <!-- Partie -->
    <template v-else>
      <div class="board-area">
        <div class="eval" :title="`Évaluation : ${evalLabel}`" :class="{ flip: orientation === 'b' }">
          <div class="eval-fill" :style="{ height: `${evalPct}%` }" />
          <span class="eval-label">{{ evalLabel }}</span>
        </div>
        <div class="board-stack">
          <div class="player top">
            <Mascot :size="38" :mood="game.thinking ? 'think' : 'happy'" />
            <div>
              <strong>Lumi</strong> <span class="chip">{{ levelInfo.name }}</span>
              <div class="captured">
                <ChessPiece v-for="(k, i) in takenBy(game.you === 'w' ? 'b' : 'w')" :key="i" :kind="k" :color="game.you" class="cap" />
                <span v-if="advantage(game.you === 'w' ? 'b' : 'w') > 0" class="adv">+{{ advantage(game.you === "w" ? "b" : "w") }}</span>
              </div>
            </div>
            <span v-if="game.thinking" class="thinking"><i /><i /><i /></span>
          </div>
          <ChessBoard
            :fen="game.fen"
            :orientation="orientation"
            :interactive="game.yourTurn && !game.thinking"
            :legal="game.position?.legal ?? []"
            :last-move="game.lastMove"
            :check="game.position?.check ? game.position.king_square : null"
            :arrows="arrows"
            :show-coords="progress.settings.coords"
            @move="game.play"
          />
          <div class="player bottom">
            <span class="you-avatar"><ChessPiece kind="k" :color="game.you" /></span>
            <div>
              <strong>Toi</strong> <span class="chip">{{ game.you === "w" ? "Blancs" : "Noirs" }}</span>
              <div class="captured">
                <ChessPiece v-for="(k, i) in takenBy(game.you)" :key="i" :kind="k" :color="game.you === 'w' ? 'b' : 'w'" class="cap" />
                <span v-if="advantage(game.you) > 0" class="adv">+{{ advantage(game.you) }}</span>
              </div>
            </div>
            <span v-if="game.yourTurn && !game.thinking" class="your-turn">À toi de jouer</span>
          </div>
        </div>
      </div>

      <aside class="side glass">
        <div class="coach">
          <Mascot :size="64" :mood="mood" />
          <div class="coach-body">
            <template v-if="coachText">
              <p class="bubble lesson-text" v-html="rich(coachText)" />
            </template>
            <template v-else-if="game.lastReview && game.lastYourPly">
              <div class="quality" :class="QUALITY[game.lastReview.quality].tone">
                <Icon :name="QUALITY[game.lastReview.quality].icon" :size="16" />
                <strong>{{ game.lastYourPly.san }}</strong> · {{ game.lastReview.label }}
              </div>
              <ul class="msgs lesson-text">
                <li v-for="(m, i) in game.lastReview.messages" :key="i" v-html="rich(m)" />
                <li v-if="!game.lastReview.messages.length">Coup joué, rien de spécial à signaler.</li>
              </ul>
              <button
                v-if="game.lastReview.best_san && game.lastReview.loss > 30"
                type="button"
                class="btn btn-ghost btn-sm best"
                @click="showBest = !showBest"
              >
                <Icon name="eye" :size="15" /> {{ showBest ? "Masquer" : "Voir" }} le meilleur coup : {{ game.lastReview.best_san }}
              </button>
            </template>
          </div>
        </div>

        <div ref="movesBox" class="moves">
          <p v-if="!rows.length" class="muted empty">Les coups de la partie apparaîtront ici.</p>
          <div v-for="r in rows" :key="r.n" class="row">
            <span class="n">{{ r.n }}.</span>
            <span v-if="r.w" class="mv" :class="r.w.review ? `q-${r.w.review.quality}` : ''">{{ r.w.san }}</span>
            <span v-if="r.b" class="mv" :class="r.b.review ? `q-${r.b.review.quality}` : ''">{{ r.b.san }}</span>
          </div>
        </div>

        <div class="controls">
          <button type="button" class="btn btn-ghost btn-sm" :disabled="!game.yourTurn || game.thinking" title="Indice" @click="game.askHint()">
            <Icon name="bulb" :size="16" /> Indice
          </button>
          <button type="button" class="btn btn-ghost btn-sm" :disabled="game.thinking || !game.plies.some((p) => p.by === 'you')" title="Annuler mon dernier coup" @click="game.undo()">
            <Icon name="undo" :size="16" /> Reprendre
          </button>
          <button type="button" class="btn btn-ghost btn-sm" title="Retourner l'échiquier" @click="flipped = !flipped">
            <Icon name="flip" :size="16" />
          </button>
          <button type="button" class="btn btn-ghost btn-sm" :disabled="!!game.outcome" title="Abandonner" @click="game.resign()">
            <Icon name="flag" :size="16" />
          </button>
          <button type="button" class="btn btn-ghost btn-sm" title="Nouvelle partie" @click="game.leave()">
            <Icon name="refresh" :size="16" />
          </button>
        </div>
      </aside>

      <Transition name="result">
        <div v-if="game.outcome" class="result-scrim">
          <div class="result glass" :class="game.outcome.result">
            <Mascot :size="100" :mood="game.outcome.result === 'win' ? 'wow' : game.outcome.result === 'loss' ? 'happy' : 'think'" />
            <p class="eyebrow">{{ game.outcome.reason }}</p>
            <h2>{{ resultTitle }}</h2>
            <p class="muted">{{ resultText }}</p>
            <div v-if="Object.keys(game.qualityCounts).length" class="summary">
              <span v-for="(n, q) in game.qualityCounts" :key="q" class="chip" :class="`q-${q}`">{{ n }} × {{ { best: "meilleur", excellent: "excellent", good: "bon", inaccuracy: "imprécision", mistake: "erreur", blunder: "gaffe" }[q] }}</span>
            </div>
            <div class="result-actions">
              <button type="button" class="btn btn-primary" @click="game.start(game.level, game.you, game.coach)"><Icon name="refresh" :size="16" /> Revanche</button>
              <button type="button" class="btn btn-ghost" @click="game.leave()">Changer de niveau</button>
              <button type="button" class="btn btn-ghost" @click="game.outcome = null">Voir l'échiquier</button>
            </div>
          </div>
        </div>
      </Transition>
    </template>
  </div>
</template>

<style scoped>
.play {
  position: relative;
  height: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 380px;
  gap: 24px;
  padding: 22px 28px;
}
/* ---------- préparation */
.setup {
  grid-column: 1 / -1;
  display: grid;
  place-items: center;
  overflow-y: auto;
}
.setup-card {
  width: min(760px, 100%);
  padding: 28px 32px;
  animation: pop-in 0.4s var(--spring);
}
.setup-head {
  display: flex;
  align-items: center;
  gap: 18px;
  margin-bottom: 8px;
}
.setup-head h1 {
  font-size: 34px;
}
.setup-card h3 {
  margin: 20px 0 10px;
  font-size: 16px;
  color: var(--text-2);
}
.levels {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 10px;
}
.level {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  border: 0;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px var(--line);
  text-align: left;
  transition: transform 0.2s var(--spring), background 0.2s, box-shadow 0.2s;
}
.level:hover {
  transform: translateY(-3px);
}
.level.on {
  background: linear-gradient(160deg, rgba(255, 79, 163, 0.3), rgba(155, 107, 255, 0.25));
  box-shadow: inset 0 0 0 2px var(--pink), 0 8px 24px rgba(255, 79, 163, 0.25);
}
.level .stars {
  display: flex;
  gap: 2px;
  color: var(--gold);
}
.level strong {
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 15px;
}
.level small {
  font-size: 11.5px;
  color: var(--text-2);
  line-height: 1.35;
}
.colors {
  display: flex;
  gap: 10px;
}
.color {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px 8px 8px;
  border: 0;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px var(--line);
  font-weight: 700;
}
.color.on {
  box-shadow: inset 0 0 0 2px var(--cyan);
  background: rgba(50, 224, 255, 0.12);
}
.swatch {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  padding: 4px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.08);
}
.dice {
  font-family: var(--font-display);
  font-size: 22px;
  color: var(--gold);
}
.toggle {
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 22px 0;
  cursor: pointer;
}
.toggle input {
  display: none;
}
.switch {
  position: relative;
  flex: none;
  width: 48px;
  height: 28px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.15);
  transition: background 0.2s;
}
.switch::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.25s var(--spring);
}
.toggle input:checked + .switch {
  background: var(--grad);
}
.toggle input:checked + .switch::after {
  transform: translateX(20px);
}
.big {
  width: 100%;
  height: 50px;
  font-size: 17px;
}

/* ---------- partie */
.board-area {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  min-height: 0;
}
.board-stack {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: min(100%, calc(100vh - 170px));
}
.player {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 46px;
  padding: 0 4px;
}
.player strong {
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 16px;
}
.player .chip {
  height: 22px;
  font-size: 11.5px;
}
.captured {
  display: flex;
  align-items: center;
  height: 20px;
}
.cap {
  width: 18px !important;
  height: 18px !important;
  margin-right: -5px;
}
.adv {
  margin-left: 10px;
  font-size: 12px;
  font-weight: 800;
  color: var(--green);
}
.you-avatar {
  width: 38px;
  height: 38px;
  padding: 3px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.08);
}
.your-turn {
  margin-left: auto;
  padding: 4px 12px;
  border-radius: 99px;
  background: var(--grad);
  font-size: 12.5px;
  font-weight: 800;
  animation: pop-in 0.3s var(--spring);
}
.thinking {
  margin-left: auto;
  display: flex;
  gap: 5px;
}
.thinking i {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--pink);
  animation: bounce 1s ease-in-out infinite;
}
.thinking i:nth-child(2) {
  animation-delay: 0.15s;
  background: var(--violet);
}
.thinking i:nth-child(3) {
  animation-delay: 0.3s;
  background: var(--cyan);
}
@keyframes bounce {
  50% {
    transform: translateY(-7px);
  }
}
.eval {
  position: relative;
  align-self: center;
  width: 16px;
  height: min(70vh, 560px);
  border-radius: 10px;
  overflow: hidden;
  background: #2a0f5c;
  box-shadow: inset 0 0 0 1px var(--line-strong);
}
.eval-fill {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(0deg, #ffffff, #ffd6f0);
  transition: height 0.6s var(--ease);
}
.eval.flip {
  transform: rotate(180deg);
}
.eval-label {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%) rotate(-90deg);
  font-size: 10px;
  font-weight: 800;
  color: var(--pink);
  mix-blend-mode: difference;
  white-space: nowrap;
}
.eval.flip .eval-label {
  transform: translate(-50%, -50%) rotate(90deg);
}

/* ---------- panneau */
.side {
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 18px;
}
.coach {
  display: flex;
  gap: 10px;
  min-height: 150px;
}
.coach :deep(.mascot) {
  flex: none;
}
.coach-body {
  flex: 1;
  min-width: 0;
}
.bubble {
  margin: 0;
  padding: 12px 14px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.95);
  color: #2c1263;
  font-weight: 700;
  line-height: 1.5;
}
.bubble.lesson-text :deep(strong) {
  color: #c2187a;
}
.quality {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 6px 12px;
  border-radius: 12px;
  font-weight: 700;
  animation: pop-in 0.3s var(--spring);
}
.quality.gold {
  background: rgba(255, 209, 102, 0.2);
  color: var(--gold);
}
.quality.green {
  background: rgba(69, 227, 160, 0.18);
  color: var(--green);
}
.quality.cyan {
  background: rgba(50, 224, 255, 0.16);
  color: var(--cyan);
}
.quality.orange {
  background: rgba(255, 159, 69, 0.18);
  color: var(--orange);
}
.quality.pink {
  background: rgba(255, 79, 163, 0.2);
  color: var(--pink);
}
.quality.red {
  background: rgba(255, 93, 115, 0.22);
  color: var(--red);
}
.msgs {
  margin: 10px 0 0;
  padding-left: 18px;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text);
}
.msgs li {
  margin-bottom: 6px;
}
.best {
  margin-top: 4px;
}
.moves {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  margin: 14px 0;
  padding: 10px;
  border-radius: 14px;
  background: rgba(0, 0, 0, 0.18);
  font-family: var(--font-display);
}
.moves .empty {
  margin: 8px;
  font-family: var(--font);
  font-size: 13px;
}
.row {
  display: grid;
  grid-template-columns: 38px 1fr 1fr;
  padding: 3px 4px;
  border-radius: 8px;
}
.row:nth-child(odd) {
  background: rgba(255, 255, 255, 0.03);
}
.n {
  color: var(--text-3);
}
.mv {
  font-weight: 500;
}
.q-best {
  color: var(--gold);
}
.q-excellent {
  color: var(--green);
}
.q-inaccuracy {
  color: var(--orange);
}
.q-mistake {
  color: var(--pink);
}
.q-blunder {
  color: var(--red);
  text-decoration: underline wavy;
}
.controls {
  display: flex;
  gap: 8px;
}
.controls .btn:nth-child(-n + 2) {
  flex: 1;
}

/* ---------- résultat */
.result-scrim {
  position: absolute;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  background: rgba(8, 0, 24, 0.55);
  backdrop-filter: blur(4px);
}
.result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  width: min(480px, 90%);
  padding: 28px;
  text-align: center;
}
.result h2 {
  font-size: 40px;
}
.result.win h2 {
  background: var(--grad);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.summary {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 6px;
  margin: 8px 0;
}
.result-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 10px;
  margin-top: 10px;
}
.result-enter-active {
  transition: opacity 0.3s;
}
.result-enter-active .result {
  animation: pop-in 0.45s var(--spring);
}
.result-enter-from {
  opacity: 0;
}
</style>
