import { defineStore } from "pinia";

export type Route =
  | { name: "home" }
  | { name: "lessons" }
  | { name: "lesson"; id: string }
  | { name: "play" }
  | { name: "coords" }
  | { name: "puzzles" }
  | { name: "puzzle"; id: string }
  | { name: "glossary" }
  | { name: "appearance" };

export const useUiStore = defineStore("ui", {
  state: () => ({
    route: { name: "home" } as Route,
    toast: null as { text: string; kind: "info" | "win" } | null,
  }),
  actions: {
    go(route: Route) {
      this.route = route;
    },
    notify(text: string, kind: "info" | "win" = "info") {
      this.toast = { text, kind };
      setTimeout(() => {
        if (this.toast?.text === text) this.toast = null;
      }, 3000);
    },
  },
});
