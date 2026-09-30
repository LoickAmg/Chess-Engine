<script setup lang="ts">
import { computed, ref } from "vue";
import { GLOSSARY } from "@/lib/content";
import ChessPiece from "@/components/ChessPiece.vue";
import Icon from "@/components/Icon.vue";
import type { PieceKind } from "@/lib/chess";

const query = ref("");
const norm = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();
const entries = computed(() => {
  const q = norm(query.value.trim());
  const sorted = [...GLOSSARY].sort((a, b) => a.term.localeCompare(b.term, "fr"));
  return q ? sorted.filter((e) => norm(e.term).includes(q) || norm(e.def).includes(q)) : sorted;
});

const PIECES: { kind: PieceKind; name: string; letter: string; value: string; move: string }[] = [
  { kind: "k", name: "Roi", letter: "R", value: "∞", move: "Une case dans toutes les directions." },
  { kind: "q", name: "Dame", letter: "D", value: "9", move: "Lignes droites et diagonales, aussi loin qu'elle veut." },
  { kind: "r", name: "Tour", letter: "T", value: "5", move: "Lignes droites (colonnes et rangées)." },
  { kind: "b", name: "Fou", letter: "F", value: "3", move: "Diagonales, toujours sur la même couleur." },
  { kind: "n", name: "Cavalier", letter: "C", value: "3", move: "Saut en L, par-dessus les pièces." },
  { kind: "p", name: "Pion", letter: "—", value: "1", move: "Avance tout droit, capture en diagonale." },
];
</script>

<template>
  <div class="page">
    <header>
      <p class="eyebrow">Aide-mémoire</p>
      <h1>Lexique</h1>
    </header>

    <section class="pieces">
      <div v-for="p in PIECES" :key="p.kind" class="piece-card glass">
        <div class="art"><ChessPiece :kind="p.kind" color="w" /></div>
        <div>
          <h3>{{ p.name }} <span class="letter">{{ p.letter }}</span></h3>
          <p>{{ p.move }}</p>
          <span class="chip">{{ p.value }} point{{ p.value === "1" ? "" : "s" }}</span>
        </div>
      </div>
    </section>

    <label class="search glass">
      <Icon name="search" :size="18" />
      <input v-model="query" type="search" placeholder="Chercher un mot : clouage, roque, pat…" />
    </label>

    <dl class="terms">
      <div v-for="e in entries" :key="e.term" class="term glass">
        <dt>{{ e.term }}</dt>
        <dd>{{ e.def }}</dd>
      </div>
      <p v-if="!entries.length" class="muted">Aucun mot ne correspond à « {{ query }} ».</p>
    </dl>
  </div>
</template>

<style scoped>
.page {
  height: 100%;
  overflow-y: auto;
  padding: 36px 48px 48px;
}
h1 {
  font-size: 44px;
  margin: 4px 0 20px;
}
.pieces {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}
.piece-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px;
}
.art {
  flex: none;
  width: 64px;
  height: 64px;
  padding: 4px;
  border-radius: 16px;
  background: linear-gradient(145deg, #6a3fc9, #3a1f86);
}
h3 {
  font-size: 18px;
}
.letter {
  display: inline-grid;
  place-items: center;
  width: 24px;
  height: 24px;
  margin-left: 4px;
  border-radius: 7px;
  background: rgba(50, 224, 255, 0.18);
  color: var(--cyan);
  font-size: 14px;
}
.piece-card p {
  margin: 4px 0 6px;
  font-size: 13px;
  color: var(--text-2);
}
.search {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 26px 0 16px;
  padding: 0 16px;
  height: 50px;
  color: var(--text-2);
}
.search input {
  flex: 1;
  border: 0;
  outline: 0;
  background: none;
  font-size: 15px;
}
.terms {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 12px;
  margin: 0;
}
.term {
  padding: 14px 16px;
}
dt {
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 17px;
  color: var(--gold);
}
dd {
  margin: 4px 0 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text);
}
</style>
