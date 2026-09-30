<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { api, type LegalMove } from "@/lib/api";
import { freeMoves, parseFen, toFen, type Arrow, type Highlight } from "@/lib/chess";
import { LESSONS, rich, type Step } from "@/lib/content";
import { sfx } from "@/lib/sound";
import { useProgressStore } from "@/stores/progress";
import { useUiStore } from "@/stores/ui";
import ChessBoard from "@/components/ChessBoard.vue";
import Icon from "@/components/Icon.vue";
import LessonIcon from "@/components/LessonIcon.vue";
import Mascot from "@/components/Mascot.vue";

const props = defineProps<{ id: string }>();
const ui = useUiStore();
const progress = useProgressStore();

const lesson = computed(() => LESSONS.find((l) => l.id === props.id)!);
const index = ref(Math.min(progress.lessonStep(props.id), lesson.value.steps.length - 1));
const step = computed<Step>(() => lesson.value.steps[index.value]);
const isLast = computed(() => index.value === lesson.value.steps.length - 1);
const finished = ref(false);
const nextLesson = computed(() => {
  const i = LESSONS.findIndex((l) => l.id === props.id);
  return LESSONS[i + 1] ?? null;
});

// --------------------------------------------------------------- état de l'étape

const EMPTY = "8/8/8/8/8/8/8/8 w - - 0 1";
const fen = ref(EMPTY);
const legal = ref<LegalMove[]>([]);
const lastMove = ref<[string, string] | null>(null);
const check = ref<string | null>(null);
const marks = ref<Highlight[]>([]);
const extraArrows = ref<Arrow[]>([]);
const stars = ref<string[]>([]);
const moves = ref(0);
const solved = ref(false);
const feedback = ref<{ kind: "good" | "bad" | "info"; text: string } | null>(null);
const picked = ref<number | null>(null);
const wrongTries = ref(0);
const squareIndex = ref(0);
const busy = ref(false);
const mood = ref<"happy" | "think" | "wow" | "sad">("happy");

const highlights = computed(() => [...(step.value.highlights ?? []), ...marks.value]);
const arrows = computed(() => [...(step.value.arrows ?? []), ...extraArrows.value]);
const orientation = computed(() => step.value.orientation ?? "w");

async function loadStep() {
  const s = step.value;
  fen.value = s.fen ?? EMPTY;
  legal.value = [];
  lastMove.value = null;
  check.value = null;
  marks.value = [];
  extraArrows.value = [];
  stars.value = s.type === "stars" ? [...s.stars] : [];
  moves.value = 0;
  feedback.value = null;
  picked.value = null;
  wrongTries.value = 0;
  squareIndex.value = 0;
  solved.value = s.type === "text";
  mood.value = s.type === "text" ? "happy" : "think";
  if (s.type === "move") {
    const info = await api.position(s.fen);
    legal.value = info.legal;
    check.value = info.check ? info.king_square : null;
  }
}

watch(index, () => {
  progress.setLessonStep(props.id, index.value);
  void loadStep();
});
onMounted(loadStep);

function succeed(text: string) {
  solved.value = true;
  feedback.value = { kind: "good", text };
  mood.value = "wow";
  sfx.success();
}
function fail(text: string) {
  feedback.value = { kind: "bad", text };
  mood.value = "sad";
  sfx.wrong();
}

// --------------------------------------------------------------- exercices

// Étoiles : la pièce se déplace librement (sans roi ni échec), comme dans les jeux d'initiation.
const mover = computed(() =>
  step.value.type === "stars" && !solved.value
    ? (from: string) => {
        const p = parseFen(fen.value).board.get(from);
        return p?.color === "w" ? freeMoves(parseFen(fen.value).board, from) : [];
      }
    : null,
);

function onStarMove(uci: string) {
  const from = uci.slice(0, 2);
  const to = uci.slice(2, 4);
  const board = parseFen(fen.value).board;
  const piece = board.get(from);
  if (!piece) return;
  board.delete(from);
  board.set(to, piece);
  fen.value = toFen(board);
  lastMove.value = [from, to];
  moves.value++;
  if (stars.value.includes(to)) {
    stars.value = stars.value.filter((s) => s !== to);
    sfx.star();
  } else sfx.move();
  if (!stars.value.length) {
    const s = step.value as Extract<Step, { type: "stars" }>;
    const par = s.stars.length;
    succeed(moves.value <= par ? `Parfait : toutes les étoiles en ${moves.value} coups !` : `Toutes les étoiles ! (${moves.value} coups — faisable en ${par})`);
  }
}

async function onMove(uci: string) {
  const s = step.value;
  if (s.type === "stars") return onStarMove(uci);
  if (s.type !== "move" || solved.value || busy.value) return;
  busy.value = true;
  try {
    const before = fen.value;
    const played = await api.play(before, uci);
    const accepted =
      s.accept.includes("*") ||
      s.accept.includes(uci) ||
      (s.accept.includes("#") && played.position.checkmate);
    fen.value = played.position.fen;
    lastMove.value = [uci.slice(0, 2), uci.slice(2, 4)];
    check.value = played.position.check ? played.position.king_square : null;
    if (played.captured) sfx.capture();
    else if (played.position.check) sfx.check();
    else sfx.move();
    if (accepted) {
      succeed(s.success);
      legal.value = [];
      return;
    }
    wrongTries.value++;
    fail(s.wrong?.[uci] ?? s.fallback ?? "Ce n'est pas le coup que je cherche. Regarde encore et réessaie !");
    // On montre le coup, puis on revient à la position de l'exercice.
    setTimeout(async () => {
      fen.value = before;
      lastMove.value = null;
      const info = await api.position(before);
      legal.value = info.legal;
      check.value = info.check ? info.king_square : null;
      mood.value = "think";
    }, 900);
  } finally {
    busy.value = false;
  }
}

async function showHint() {
  const s = step.value;
  if (s.type === "move") {
    const explicit = s.accept.find((u) => u !== "*" && u !== "#");
    const uci = explicit ?? (await api.hint(fen.value))?.uci;
    if (uci) {
      extraArrows.value = [{ from: uci.slice(0, 2), to: uci.slice(2, 4), color: "gold" }];
      feedback.value = { kind: "info", text: "Regarde la flèche dorée…" };
    }
  } else if (s.type === "square") {
    marks.value = [{ sq: s.targets[squareIndex.value], kind: "focus" }];
  }
}

function onSquare(sq: string) {
  const s = step.value;
  if (s.type !== "square" || solved.value) return;
  const target = s.targets[squareIndex.value];
  if (sq === target) {
    marks.value = [...marks.value.filter((m) => m.kind === "good"), { sq, kind: "good" }];
    squareIndex.value++;
    if (squareIndex.value >= s.targets.length) succeed(`Oui ! C'est bien **${sq}**.`);
    else sfx.star();
  } else {
    marks.value = [...marks.value.filter((m) => m.kind === "good"), { sq, kind: "bad" }];
    fail(`Non, ça c'est **${sq}**. Cherche la colonne **${target[0]}**, puis la rangée **${target[1]}**.`);
  }
}

function choose(i: number) {
  const s = step.value;
  if (s.type !== "quiz" || solved.value) return;
  picked.value = i;
  if (i === s.answer) succeed(s.explain);
  else {
    wrongTries.value++;
    fail(wrongTries.value >= 2 ? `Pas tout à fait. ${s.explain}` : "Pas tout à fait… réfléchis encore !");
  }
}

// --------------------------------------------------------------- navigation

function next() {
  if (!solved.value) return;
  if (isLast.value) {
    progress.completeLesson(props.id);
    finished.value = true;
    sfx.fanfare();
    return;
  }
  index.value++;
}
function prev() {
  if (index.value > 0) index.value--;
}
function onKey(e: KeyboardEvent) {
  if ((e.key === "Enter" || e.key === "ArrowRight") && solved.value && !finished.value) next();
  if (e.key === "ArrowLeft") prev();
}
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));

const instruction = computed(() => {
  const s = step.value;
  if (s.type === "square") return s.targets.length > 1 ? `Case ${squareIndex.value + 1} / ${s.targets.length}` : "Clique sur la bonne case";
  if (s.type === "stars") return `${stars.value.length} étoile${stars.value.length > 1 ? "s" : ""} restante${stars.value.length > 1 ? "s" : ""} · ${moves.value} coup${moves.value > 1 ? "s" : ""}`;
  if (s.type === "move") return "Joue le coup sur l'échiquier";
  if (s.type === "quiz") return "Choisis la bonne réponse";
  return "";
});
</script>

<template>
  <div class="lesson">
    <div class="board-col">
      <ChessBoard
        :fen="fen"
        :orientation="orientation"
        :interactive="(step.type === 'move' || step.type === 'stars') && !solved"
        :legal="legal"
        :mover="mover"
        :last-move="lastMove"
        :check="check"
        :highlights="highlights"
        :arrows="arrows"
        :stars="stars"
        :square-names="step.type === 'text' && !!step.squareNames"
        :pick-squares="step.type === 'square' && !solved"
        @move="onMove"
        @square="onSquare"
      />
    </div>

    <aside class="panel glass">
      <header class="top">
        <button type="button" class="icon-btn" title="Retour aux leçons" @click="ui.go({ name: 'lessons' })">
          <Icon name="back" :size="20" />
        </button>
        <LessonIcon :icon="lesson.icon" :size="40" />
        <div class="titles">
          <p class="eyebrow">{{ lesson.chapter }}</p>
          <h1>{{ lesson.title }}</h1>
        </div>
      </header>
      <div class="dots" role="progressbar" :aria-valuenow="index + 1" :aria-valuemax="lesson.steps.length">
        <span v-for="(_s, i) in lesson.steps" :key="i" :class="{ on: i < index || (i === index && solved), cur: i === index }" />
      </div>

      <Transition name="swap" mode="out-in">
        <div v-if="!finished" :key="index" class="body">
          <div class="coach">
            <Mascot :size="70" :mood="mood" />
            <p class="bubble lesson-text" v-html="rich(step.text)" />
          </div>

          <p v-if="instruction" class="instruction"><Icon name="sparkle" :size="14" /> {{ instruction }}</p>

          <div v-if="step.type === 'quiz'" class="choices">
            <button
              v-for="(c, i) in step.choices"
              :key="i"
              type="button"
              class="choice"
              :class="{ right: solved && i === step.answer, wrong: picked === i && i !== step.answer }"
              :disabled="solved"
              @click="choose(i)"
            >
              <span class="letter">{{ "ABCD"[i] }}</span>{{ c }}
            </button>
          </div>

          <Transition name="fb">
            <p v-if="feedback" :key="feedback.text" class="feedback lesson-text" :class="feedback.kind" v-html="rich(feedback.text)" />
          </Transition>
        </div>
        <div v-else class="body done-card">
          <Mascot :size="110" mood="wow" />
          <h2>Leçon terminée !</h2>
          <p class="muted">« {{ lesson.title }} » est dans ta poche. Bravo !</p>
          <div class="done-actions">
            <button v-if="nextLesson" type="button" class="btn btn-primary" @click="ui.go({ name: 'lesson', id: nextLesson.id })">
              Leçon suivante : {{ nextLesson.title }} <Icon name="next" :size="16" />
            </button>
            <button type="button" class="btn btn-ghost" @click="ui.go({ name: 'lessons' })">Toutes les leçons</button>
          </div>
        </div>
      </Transition>

      <footer v-if="!finished" class="actions">
        <button type="button" class="btn btn-ghost btn-sm" :disabled="index === 0" @click="prev"><Icon name="back" :size="16" /> Précédent</button>
        <button
          v-if="(step.type === 'move' || step.type === 'square') && !solved"
          type="button"
          class="btn btn-ghost btn-sm hint"
          @click="showHint"
        >
          <Icon name="bulb" :size="16" /> Indice
        </button>
        <button type="button" class="btn btn-primary" :disabled="!solved" @click="next">
          {{ isLast ? "Terminer" : "Suivant" }} <Icon name="next" :size="16" />
        </button>
      </footer>
    </aside>
  </div>
</template>

<style scoped>
.lesson {
  height: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 420px;
  gap: 28px;
  padding: 28px 32px;
  align-items: center;
}
.board-col {
  display: grid;
  place-items: center;
  height: 100%;
  min-height: 0;
}
.board-col > :deep(.board-wrap) {
  width: min(100%, calc(100vh - 80px));
}
.panel {
  height: min(100%, 760px);
  display: flex;
  flex-direction: column;
  padding: 20px 22px;
}
.top {
  display: flex;
  align-items: center;
  gap: 12px;
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
.icon-btn:hover {
  background: rgba(255, 255, 255, 0.12);
}
.titles h1 {
  font-size: 24px;
  line-height: 1.1;
}
.titles .eyebrow {
  font-size: 11px;
}
.dots {
  display: flex;
  gap: 5px;
  margin: 16px 0 18px;
}
.dots span {
  flex: 1;
  height: 6px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  transition: background 0.3s;
}
.dots span.on {
  background: var(--grad);
}
.dots span.cur:not(.on) {
  background: rgba(255, 79, 163, 0.45);
}
.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.coach {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.coach :deep(.mascot) {
  flex: none;
  margin-top: 6px;
}
.bubble {
  position: relative;
  margin: 0;
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.95);
  color: #2c1263;
  font-size: 15px;
  font-weight: 600;
  line-height: 1.55;
  box-shadow: 0 10px 30px rgba(10, 0, 30, 0.4);
}
.bubble.lesson-text :deep(strong) {
  color: #c2187a;
}
.bubble.lesson-text :deep(em) {
  color: #0e7fa6;
}
.bubble::before {
  content: "";
  position: absolute;
  left: -7px;
  top: 26px;
  width: 14px;
  height: 14px;
  background: inherit;
  transform: rotate(45deg);
  border-radius: 3px;
}
.instruction {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 16px 0 0;
  font-size: 13px;
  font-weight: 800;
  color: var(--cyan);
}
.choices {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 14px;
}
.choice {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 14px;
  border: 0;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 0 0 1px var(--line);
  text-align: left;
  font-weight: 700;
  transition: background 0.2s, transform 0.2s var(--spring), box-shadow 0.2s;
}
.choice:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  transform: translateX(4px);
}
.choice:disabled {
  cursor: default;
}
.letter {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border-radius: 9px;
  background: rgba(155, 107, 255, 0.35);
  font-family: var(--font-display);
  font-size: 14px;
}
.choice.right {
  background: rgba(69, 227, 160, 0.22);
  box-shadow: inset 0 0 0 2px var(--green);
}
.choice.wrong {
  background: rgba(255, 93, 115, 0.2);
  box-shadow: inset 0 0 0 2px var(--red);
  animation: shake 0.4s;
}
@keyframes shake {
  25% {
    transform: translateX(-6px);
  }
  75% {
    transform: translateX(6px);
  }
}
.feedback {
  margin: 16px 0 0;
  padding: 12px 14px;
  border-radius: 14px;
  font-weight: 700;
  line-height: 1.5;
}
.feedback.good {
  background: rgba(69, 227, 160, 0.16);
  box-shadow: inset 0 0 0 1px rgba(69, 227, 160, 0.5);
}
.feedback.bad {
  background: rgba(255, 93, 115, 0.16);
  box-shadow: inset 0 0 0 1px rgba(255, 93, 115, 0.5);
}
.feedback.info {
  background: rgba(255, 209, 102, 0.14);
  box-shadow: inset 0 0 0 1px rgba(255, 209, 102, 0.45);
}
.actions {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-top: 14px;
  border-top: 1px solid var(--line);
}
.actions .btn-primary {
  margin-left: auto;
}
.done-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  text-align: center;
}
.done-card h2 {
  font-size: 30px;
}
.done-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 10px;
}
.swap-enter-active,
.swap-leave-active {
  transition: opacity 0.2s, transform 0.25s var(--ease);
}
.swap-enter-from {
  opacity: 0;
  transform: translateX(16px);
}
.swap-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
.fb-enter-active {
  animation: pop-in 0.3s var(--spring);
}
</style>
