// Partie toujours en cours sur le plateau 3D de l'accueil : alternance d'une partie
// célèbre (rejouée coup par coup, données vérifiées par le moteur) et d'une partie du
// moteur contre lui-même. Elle se met en pause quand l'accueil n'est pas visible.

import famousGames from "@/data/famous-games.json";
import { api } from "@/lib/api";
import { parseFen, START_FEN } from "@/lib/chess";
import type { BoardScene } from "./scene";

interface FamousGame {
  title: string;
  players: string;
  place: string;
  uci: string[];
  san: string[];
}

export interface DemoStatus {
  kind: "famous" | "engine";
  title: string;
  subtitle: string;
  /** Dernier coup joué (« 17. Td8# »), ou résultat en fin de partie. */
  move: string;
}

const GAMES = famousGames as FamousGame[];
const MOVE_GAP = 1100; // pause entre deux coups
const END_PAUSE = 3800; // position finale montrée avant la partie suivante
const MAX_PLIES = 160;

function moveLabel(ply: number, san: string) {
  const n = Math.floor(ply / 2) + 1;
  return ply % 2 === 0 ? `${n}. ${san}` : `${n}… ${san}`;
}

export class DemoGames {
  private stopped = false;
  private engineOk: boolean | null = null;

  constructor(
    private scene: BoardScene,
    private onStatus: (status: DemoStatus) => void,
    /** Se résout quand le plateau est de nouveau visible (accueil affiché, appli au premier plan). */
    private waitVisible: () => Promise<void>,
  ) {}

  stop() {
    this.stopped = true;
  }

  private async sleep(ms: number) {
    await new Promise((r) => setTimeout(r, ms));
    await this.waitVisible();
  }

  /** Le moteur répond-il (application de bureau, ou passerelle de développement) ? */
  private async engineAvailable(): Promise<boolean> {
    if (this.engineOk !== null) return this.engineOk;
    try {
      await Promise.race([
        api.position(START_FEN),
        new Promise((_, reject) => setTimeout(() => reject(new Error("délai")), 2000)),
      ]);
      this.engineOk = true;
    } catch {
      this.engineOk = false;
    }
    return this.engineOk;
  }

  async run() {
    let famous = Math.floor(Math.random() * GAMES.length);
    let engineTurn = false;
    while (!this.stopped) {
      this.scene.setPosition(parseFen(START_FEN).board);
      await this.sleep(1200);
      if (this.stopped) return;
      if (engineTurn && (await this.engineAvailable())) await this.playEngine();
      else await this.playFamous(GAMES[famous++ % GAMES.length]);
      engineTurn = !engineTurn;
      if (this.stopped) return;
      await this.sleep(END_PAUSE);
      if (this.stopped) return;
      await this.scene.clear();
    }
  }

  private async playFamous(game: FamousGame) {
    const subtitle = `${game.players} · ${game.place}`;
    this.onStatus({ kind: "famous", title: game.title, subtitle, move: "Position de départ" });
    for (let ply = 0; ply < game.uci.length && !this.stopped; ply++) {
      await this.scene.move(game.uci[ply]);
      this.onStatus({ kind: "famous", title: game.title, subtitle, move: moveLabel(ply, game.san[ply]) });
      await this.sleep(MOVE_GAP);
    }
  }

  private async playEngine() {
    const level = 3 + Math.floor(Math.random() * 3);
    const title = "Academy contre Academy";
    const subtitle = `Le moteur de l'application joue contre lui-même (niveau ${level})`;
    this.onStatus({ kind: "engine", title, subtitle, move: "Position de départ" });
    let fen = START_FEN;
    for (let ply = 0; ply < MAX_PLIES && !this.stopped; ply++) {
      let result: string | null = null;
      try {
        const reply = await api.engineReply(fen, level);
        if (!reply) break;
        const played = await api.play(fen, reply.uci);
        await this.scene.move(reply.uci);
        fen = played.position.fen;
        const p = played.position;
        if (p.checkmate) result = "Échec et mat";
        else if (p.stalemate) result = "Pat : partie nulle";
        else if (p.insufficient_material) result = "Matériel insuffisant : partie nulle";
        this.onStatus({
          kind: "engine",
          title,
          subtitle,
          move: moveLabel(ply, played.san) + (result ? ` — ${result}` : ""),
        });
      } catch {
        this.engineOk = false; // moteur indisponible : parties célèbres seulement
        return;
      }
      if (result) return;
      await this.sleep(MOVE_GAP);
    }
  }
}
