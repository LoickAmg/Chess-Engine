<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import { squareName, type Color, type Highlight } from "@/lib/chess";
import { sfx } from "@/lib/sound";
import { useProgressStore } from "@/stores/progress";
import ChessBoard from "@/components/ChessBoard.vue";
import Icon from "@/components/Icon.vue";
import Mascot from "@/components/Mascot.vue";

// Entraînement au nom des cases, contre la montre : « Trouve la case » (clic) ou
// « Nomme la case » (choix parmi quatre propositions).

const progress = useProgressStore();
type Mode = "find" | "name";
const DURATION = 30;

const mode = ref<Mode>("find");
const orientation = ref<Color>("w");
const running = ref(false);
const over = ref(false);
const timeLeft = ref(DURATION);
const score = ref(0);
const mistakes = ref(0);
const target = ref("e4");
const choices = ref<string[]>([]);
const marks = ref<Highlight[]>([]);
const newRecord = ref(false);
let timer: ReturnType<typeof setInterval> | null = null;

const randomSquare = () => squareName(Math.floor(Math.random() * 8), Math.floor(Math.random() * 8));

function nextTarget() {
  let sq = randomSquare();
  while (sq === target.value) sq = randomSquare();
  target.value = sq;
  if (mode.value === "name") {
    const set = new Set([sq]);
    // Des pièges crédibles : même colonne, même rangée, ou case « miroir ».
    const near = [
      `${sq[0]}${9 - Number(sq[1])}`,
      `${String.fromCharCode(201 - sq.charCodeAt(0))}${sq[1]}`,
      `${sq[0]}${((Number(sq[1]) + 1) % 8) + 1}`,
    ];
    for (const n of near) if (set.size < 4) set.add(n);
    while (set.size < 4) set.add(randomSquare());
    choices.value = [...set].sort(() => Math.random() - 0.5);
    marks.value = [{ sq, kind: "focus" }];
  } else marks.value = [];
}

function startGame(m: Mode) {
  mode.value = m;
  score.value = 0;
  mistakes.value = 0;
  timeLeft.value = DURATION;
  over.value = false;
  newRecord.value = false;
  running.value = true;
  nextTarget();
  if (timer) clearInterval(timer);
  const t0 = performance.now();
  timer = setInterval(() => {
    timeLeft.value = Math.max(0, DURATION - (performance.now() - t0) / 1000);
    if (timeLeft.value <= 0) stop();
  }, 100);
}

function stop() {
  if (timer) clearInterval(timer);
  timer = null;
  running.value = false;
  over.value = true;
  newRecord.value = progress.recordCoords(mode.value, score.value);
  if (newRecord.value) sfx.fanfare();
}
onBeforeUnmount(() => timer && clearInterval(timer));

function answer(sq: string) {
  if (!running.value) return;
  if (sq === target.value) {
    score.value++;
    sfx.star();
    marks.value = [{ sq, kind: "good" }];
    const done = sq;
    setTimeout(() => {
      if (marks.value[0]?.sq === done && mode.value === "find") marks.value = [];
    }, 250);
    nextTarget();
  } else {
    mistakes.value++;
    sfx.wrong();
    if (mode.value === "find") marks.value = [{ sq, kind: "bad" }, { sq: target.value, kind: "info" }];
  }
}

const best = computed(() => progress.coordsBest[mode.value] ?? 0);
const verdict = computed(() => {
  const s = score.value;
  if (s >= 25) return "Incroyable ! Tu connais l'échiquier comme ta poche.";
  if (s >= 18) return "Excellent ! Les cases n'ont presque plus de secrets pour toi.";
  if (s >= 10) return "Bien joué ! Encore quelques parties et ce sera automatique.";
  return "C'est un bon début. Astuce : lis d'abord la colonne (lettre), puis la rangée (chiffre).";
});
</script>

<template>
  <div class="coords">
    <div class="board-col">
      <ChessBoard
        fen="8/8/8/8/8/8/8/8 w - - 0 1"
        :orientation="orientation"
        :pick-squares="running && mode === 'find'"
        :highlights="marks"
        :show-coords="!running || mode === 'name' ? true : progress.settings.coords"
        @square="answer"
      />
    </div>
    <aside class="panel glass">
      <p class="eyebrow">Entraînement</p>
      <h1>Le nom des cases</h1>

      <template v-if="running">
        <div class="hud">
          <div class="timer" :class="{ low: timeLeft < 6 }">
            <Icon name="timer" :size="18" /> {{ Math.ceil(timeLeft) }} s
            <span class="bar"><span :style="{ width: `${(timeLeft / DURATION) * 100}%` }" /></span>
          </div>
          <div class="score"><strong>{{ score }}</strong> trouvées</div>
        </div>
        <div v-if="mode === 'find'" class="prompt">
          <span>Clique sur</span>
          <strong :key="target" class="target">{{ target }}</strong>
        </div>
        <div v-else class="prompt">
          <span>Comment s'appelle la case qui brille ?</span>
          <div class="answers">
            <button v-for="c in choices" :key="c + target" type="button" class="answer" @click="answer(c)">{{ c }}</button>
          </div>
        </div>
        <button type="button" class="btn btn-ghost btn-sm" @click="stop">Arrêter</button>
      </template>

      <template v-else>
        <div v-if="over" class="result">
          <Mascot :size="80" :mood="newRecord ? 'wow' : 'happy'" />
          <div>
            <p class="big-score"><strong>{{ score }}</strong> cases <span v-if="newRecord" class="chip record">Nouveau record !</span></p>
            <p class="muted">{{ mistakes }} erreur{{ mistakes > 1 ? "s" : "" }} · record : {{ best }}</p>
            <p>{{ verdict }}</p>
          </div>
        </div>
        <p v-else class="muted intro">30 secondes pour trouver un maximum de cases. Le meilleur moyen de lire et de noter les parties sans effort !</p>

        <div class="modes">
          <button type="button" class="mode" @click="startGame('find')">
            <Icon name="target" :size="24" />
            <strong>Trouve la case</strong>
            <small>Je dis « e4 », tu cliques dessus. Record : {{ progress.coordsBest.find ?? 0 }}</small>
          </button>
          <button type="button" class="mode" @click="startGame('name')">
            <Icon name="search" :size="24" />
            <strong>Nomme la case</strong>
            <small>Une case brille, tu choisis son nom. Record : {{ progress.coordsBest.name ?? 0 }}</small>
          </button>
        </div>
        <label class="side-pick">
          <span>Voir l'échiquier côté</span>
          <button type="button" class="chip" :class="{ on: orientation === 'w' }" @click="orientation = 'w'">Blancs</button>
          <button type="button" class="chip" :class="{ on: orientation === 'b' }" @click="orientation = 'b'">Noirs</button>
        </label>
      </template>
    </aside>
  </div>
</template>

<style scoped>
.coords {
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
  gap: 16px;
  padding: 24px;
}
h1 {
  font-size: 30px;
  margin-top: -8px;
}
.hud {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.timer {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 600;
}
.timer.low {
  color: var(--red);
  animation: pulse 0.6s infinite;
}
@keyframes pulse {
  50% {
    opacity: 0.6;
  }
}
.bar {
  flex: 1;
  height: 6px;
  border-radius: 6px;
  background: var(--surface-2);
  overflow: hidden;
}
.bar span {
  display: block;
  height: 100%;
  background: var(--grad);
}
.score {
  font-family: var(--font-display);
}
.score strong {
  font-size: 28px;
  color: var(--gold);
}
.prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px;
  border-radius: 18px;
  background: var(--surface-2);
  font-weight: 700;
  color: var(--text-2);
}
.target {
  font-family: var(--font-display);
  font-size: 84px;
  line-height: 1;
  font-weight: 700;
  background: var(--grad);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: pop-in 0.25s var(--spring);
}
.answers {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  width: 100%;
  margin-top: 6px;
}
.answer {
  height: 60px;
  border: 0;
  border-radius: 14px;
  background: var(--surface-2);
  box-shadow: inset 0 0 0 1px var(--line-strong);
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 600;
  animation: pop-in 0.25s var(--spring);
  transition: background 0.15s, transform 0.15s var(--spring);
}
.answer:hover {
  background: color-mix(in srgb, var(--accent-2) 18%, transparent);
  transform: translateY(-2px);
}
.modes {
  display: grid;
  gap: 10px;
}
.mode {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto auto;
  column-gap: 14px;
  align-items: center;
  padding: 16px;
  border: 0;
  border-radius: 16px;
  background: var(--surface-2);
  box-shadow: inset 0 0 0 1px var(--line);
  text-align: left;
  color: var(--cyan);
  transition: transform 0.2s var(--spring), background 0.2s;
}
.mode:hover {
  transform: translateX(4px);
  background: var(--surface-2);
}
.mode :first-child {
  grid-row: span 2;
}
.mode strong {
  color: var(--text);
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 17px;
}
.mode small {
  color: var(--text-2);
}
.result {
  display: flex;
  gap: 14px;
  align-items: center;
}
.big-score {
  margin: 0;
  font-family: var(--font-display);
  font-size: 20px;
}
.big-score strong {
  font-size: 42px;
  color: var(--gold);
}
.record {
  background: var(--grad);
  color: #fff;
}
.result p {
  margin: 4px 0;
}
.intro {
  margin: 0;
}
.side-pick {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-2);
}
.side-pick .chip {
  border: 0;
  cursor: pointer;
}
.side-pick .chip.on {
  background: color-mix(in srgb, var(--accent-2) 20%, transparent);
  color: var(--cyan);
}
</style>
