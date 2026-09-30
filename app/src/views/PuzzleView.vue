<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, type LegalMove } from "@/lib/api";
import type { Arrow, Color, Highlight } from "@/lib/chess";
import { PUZZLES, rich } from "@/lib/content";
import { sfx } from "@/lib/sound";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import ChessBoard from "@/components/ChessBoard.vue";
import Icon from "@/components/Icon.vue";
import Mascot from "@/components/Mascot.vue";

const props = defineProps<{ id: string }>();
const ui = useUiStore();
const progress = useProgressStore();

const index = computed(() => PUZZLES.findIndex((p) => p.id === props.id));
const puzzle = computed(() => PUZZLES[index.value]);
const next = computed(() => PUZZLES[index.value + 1] ?? null);
const side = computed<Color>(() => (puzzle.value.fen.split(" ")[1] === "b" ? "b" : "w"));

const fen = ref(puzzle.value.fen);
const legal = ref<LegalMove[]>([]);
const lastMove = ref<[string, string] | null>(null);
const check = ref<string | null>(null);
const step = ref(0); // index dans la solution
const state = ref<"play" | "solved" | "wrong">("play");
const message = ref("");
const arrows = ref<Arrow[]>([]);
const marks = ref<Highlight[]>([]);
const failed = ref(false);
const busy = ref(false);

async function load(f: string) {
  const info = await api.position(f);
  fen.value = info.fen;
  legal.value = info.legal;
  check.value = info.check ? info.king_square : null;
}

async function reset() {
  step.value = 0;
  state.value = "play";
  message.value = "";
  arrows.value = [];
  marks.value = [];
  lastMove.value = null;
  await load(puzzle.value.fen);
}
onMounted(reset);

async function onMove(uci: string) {
  if (state.value !== "play" || busy.value) return;
  busy.value = true;
  const before = fen.value;
  try {
    const sol = puzzle.value.solution;
    const played = await api.play(before, uci);
    lastMove.value = [uci.slice(0, 2), uci.slice(2, 4)];
    fen.value = played.position.fen;
    check.value = played.position.check ? played.position.king_square : null;
    played.captured ? sfx.capture() : played.position.check ? sfx.check() : sfx.move();

    const good = sol[0] === "#" ? played.position.checkmate : sol[step.value] === uci || (step.value === sol.length - 1 && played.position.checkmate);
    if (!good) {
      failed.value = true;
      state.value = "wrong";
      message.value = "Ce n'est pas le bon coup. Observe encore la position !";
      marks.value = [{ sq: uci.slice(2, 4), kind: "bad" }];
      sfx.wrong();
      setTimeout(async () => {
        state.value = "play";
        marks.value = [];
        lastMove.value = null;
        await load(before);
      }, 1000);
      return;
    }
    step.value++;
    if (sol[0] === "#" || step.value >= sol.length) {
      state.value = "solved";
      message.value = played.position.checkmate ? `**${played.san}** : échec et mat ! Bravo !` : `**${played.san}** : exactement ! Bien trouvé.`;
      progress.recordPuzzle(puzzle.value.id, true);
      if (!failed.value) message.value += " Du premier coup !";
      sfx.success();
      legal.value = [];
      return;
    }
    // Réponse de l'adversaire prévue par la solution
    message.value = `Bien joué ! Je réponds…`;
    const reply = sol[step.value];
    await new Promise((r) => setTimeout(r, 500));
    const answered = await api.play(fen.value, reply);
    lastMove.value = [reply.slice(0, 2), reply.slice(2, 4)];
    answered.captured ? sfx.capture() : sfx.move();
    step.value++;
    message.value = `Réponse : **${answered.san}**. Continue, trouve la suite !`;
    await load(answered.position.fen);
  } finally {
    busy.value = false;
  }
}

async function hint() {
  const sol = puzzle.value.solution;
  message.value = puzzle.value.hint;
  const uci = sol[0] === "#" ? (await api.hint(fen.value))?.uci : sol[step.value];
  if (uci) marks.value = [{ sq: uci.slice(0, 2), kind: "focus" }];
  failed.value = true;
}

async function reveal() {
  const sol = puzzle.value.solution;
  const uci = sol[0] === "#" ? (await api.hint(fen.value))?.uci : sol[step.value];
  if (uci) arrows.value = [{ from: uci.slice(0, 2), to: uci.slice(2, 4), color: "green" }];
  failed.value = true;
}
</script>

<template>
  <div class="puzzle">
    <div class="board-col">
      <ChessBoard
        :fen="fen"
        :orientation="side"
        :interactive="state === 'play'"
        :legal="legal"
        :last-move="lastMove"
        :check="check"
        :arrows="arrows"
        :highlights="marks"
        :show-coords="progress.settings.coords"
        @move="onMove"
      />
    </div>
    <aside class="panel glass">
      <header class="top">
        <button type="button" class="icon-btn" title="Tous les puzzles" @click="ui.go({ name: 'puzzles' })"><Icon name="back" :size="20" /></button>
        <div>
          <p class="eyebrow">Puzzle {{ index + 1 }} / {{ PUZZLES.length }} · {{ puzzle.theme }}</p>
          <h1>{{ puzzle.title }}</h1>
        </div>
      </header>

      <div class="goal">
        <span class="turn" :class="side" />
        <strong>{{ side === "w" ? "Les Blancs jouent" : "Les Noirs jouent" }}</strong>
        <span class="chip">{{ puzzle.theme }}</span>
      </div>

      <div class="coach">
        <Mascot :size="70" :mood="state === 'solved' ? 'wow' : state === 'wrong' ? 'sad' : 'think'" />
        <p class="bubble lesson-text" v-html="rich(message || 'Trouve le meilleur coup. Prends ton temps, regarde toutes les captures et tous les échecs !')" />
      </div>

      <div class="actions">
        <template v-if="state !== 'solved'">
          <button type="button" class="btn btn-ghost btn-sm" @click="hint"><Icon name="bulb" :size="16" /> Indice</button>
          <button type="button" class="btn btn-ghost btn-sm" @click="reveal"><Icon name="eye" :size="16" /> Solution</button>
          <button type="button" class="btn btn-ghost btn-sm" @click="reset"><Icon name="refresh" :size="16" /></button>
        </template>
        <template v-else>
          <button v-if="next" type="button" class="btn btn-primary" @click="ui.go({ name: 'puzzle', id: next.id })">Puzzle suivant <Icon name="next" :size="16" /></button>
          <button type="button" class="btn btn-ghost" @click="reset"><Icon name="refresh" :size="16" /> Rejouer</button>
        </template>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.puzzle {
  height: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 400px;
  gap: 28px;
  padding: 28px 32px;
  align-items: center;
}
.board-col {
  display: grid;
  place-items: center;
}
.board-col > .board-wrap {
  width: min(100%, calc(100vh - 80px));
}
.panel {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px;
}
.top {
  display: flex;
  align-items: center;
  gap: 12px;
}
.top h1 {
  font-size: 26px;
}
.top .eyebrow {
  font-size: 11px;
}
.icon-btn {
  display: grid;
  place-items: center;
  width: 36px;
  height: 36px;
  border: 0;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.06);
}
.goal {
  display: flex;
  align-items: center;
  gap: 10px;
}
.turn {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  box-shadow: 0 0 0 2px var(--line-strong);
}
.turn.w {
  background: #fff;
}
.turn.b {
  background: #2d1060;
}
.coach {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}
.coach :deep(.mascot) {
  flex: none;
}
.bubble {
  margin: 0;
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.95);
  color: #2c1263;
  font-weight: 700;
  line-height: 1.5;
}
.bubble.lesson-text :deep(strong) {
  color: #c2187a;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
</style>
