// Plateau d'échecs en 3D (Three.js) de l'accueil : il tourne en permanence sur lui-même,
// se retourne entièrement de temps en temps (360°), se manipule au doigt ou à la souris
// dans tous les sens (retour en douceur ensuite), et anime les coups d'une partie : la pièce
// saute en arc jusqu'à sa case, les prises disparaissent, roque, prise en passant et
// promotion compris.

import * as THREE from "three";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import { FILES, type BoardMap, type PieceKind } from "@/lib/chess";
import type { BoardId, PieceSetId } from "@/lib/themes";
import { boardMaterials, pieceMaterials, type BoardMaterials, type PieceMaterials } from "./materials";
import { pieceGeometries } from "./pieces";

const BASE_TILT = 0.62; // inclinaison de repos : le dessus du plateau face à l'œil
const SPIN_SPEED = (Math.PI * 2) / 48; // un tour complet en 48 s
const ROLL_EVERY = 38_000; // un retournement complet toutes les 38 s
const ROLL_DURATION = 7_000;
const MOVE_DURATION = 650;

/** Centre d'une case (« e4 ») dans le repère du plateau ; rang 1 côté blancs (+z). */
function squarePos(sq: string): THREE.Vector3 {
  const f = FILES.indexOf(sq[0] as (typeof FILES)[number]);
  const r = Number(sq[1]);
  return new THREE.Vector3(f - 3.5, 0, 4.5 - r);
}

const ease = (k: number) => (k < 0.5 ? 4 * k * k * k : 1 - Math.pow(-2 * k + 2, 3) / 2);

interface Tween {
  start: number;
  duration: number;
  update: (k: number) => void;
  done: () => void;
}

export class BoardScene {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(30, 1, 0.1, 200);
  private root = new THREE.Group();
  private boardGroup = new THREE.Group();
  private pieceLayer = new THREE.Group();
  private pieces = new Map<string, THREE.Group>();
  private boardMat: BoardMaterials | null = null;
  private pieceMat: PieceMaterials | null = null;
  private tweens: Tween[] = [];
  private frame = 0;
  private running = false;
  private last = 0;
  private reducedMotion: boolean;

  // Orientation = manipulation de l'utilisateur × retournement × inclinaison × rotation.
  private userQ = new THREE.Quaternion();
  private spin = 0;
  private roll = 0;
  private rollStart = 0;
  private nextRoll = 0;
  private dragging = false;
  private lastPointer = { x: 0, y: 0 };
  private idleSince = 0;

  constructor(private canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true, powerPreference: "high-performance" });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.toneMapping = THREE.NeutralToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.reducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;

    const pmrem = new THREE.PMREMGenerator(this.renderer);
    this.scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
    pmrem.dispose();

    const key = new THREE.DirectionalLight("#ffffff", 2.3);
    key.position.set(6, 12, 8);
    key.castShadow = true;
    key.shadow.mapSize.set(1024, 1024);
    key.shadow.camera.left = key.shadow.camera.bottom = -7;
    key.shadow.camera.right = key.shadow.camera.top = 7;
    key.shadow.bias = -0.0004;
    this.scene.add(key, new THREE.HemisphereLight("#dfe8ff", "#3a2a20", 0.55));

    this.camera.position.set(0, 0, 26);
    this.camera.lookAt(0, 0, 0);
    this.root.add(this.boardGroup, this.pieceLayer);
    this.scene.add(this.root);
    this.applyOrientation();

    canvas.addEventListener("pointerdown", this.onDown);
    canvas.addEventListener("pointermove", this.onMove);
    canvas.addEventListener("pointerup", this.onUp);
    canvas.addEventListener("pointercancel", this.onUp);
  }

  // ------------------------------------------------------------------ apparence

  setTheme(board: BoardId, set: PieceSetId) {
    this.boardMat?.squares.dispose();
    this.boardMat?.frame.dispose();
    this.boardMat = boardMaterials(board);
    this.boardGroup.clear();
    // Cases (dessus) et corps du plateau (cadre, tranche et dessous visibles au retournement).
    const top = new THREE.Mesh(new THREE.PlaneGeometry(8, 8), this.boardMat.squares);
    top.rotation.x = -Math.PI / 2;
    top.position.y = 0.001;
    top.receiveShadow = true;
    const body = new THREE.Mesh(new THREE.BoxGeometry(9, 0.36, 9), this.boardMat.frame);
    body.position.y = -0.18;
    body.receiveShadow = body.castShadow = true;
    this.boardGroup.add(top, body);

    const old = this.pieceMat;
    this.pieceMat = pieceMaterials(set);
    for (const group of this.pieces.values()) {
      const white = group.userData.color === "w";
      group.traverse((o) => {
        if (o instanceof THREE.Mesh) o.material = white ? this.pieceMat!.white : this.pieceMat!.black;
      });
    }
    old?.white.dispose();
    old?.black.dispose();
  }

  private makePiece(kind: PieceKind, color: "w" | "b"): THREE.Group {
    const group = new THREE.Group();
    const material = color === "w" ? this.pieceMat!.white : this.pieceMat!.black;
    for (const geo of pieceGeometries(kind)) {
      const mesh = new THREE.Mesh(geo, material);
      mesh.castShadow = mesh.receiveShadow = true;
      group.add(mesh);
    }
    if (color === "b") group.rotation.y = Math.PI; // les cavaliers noirs regardent les blancs
    group.userData = { kind, color };
    return group;
  }

  /** Place toutes les pièces d'une position (sans animation). */
  setPosition(board: BoardMap) {
    this.pieceLayer.clear();
    this.pieces.clear();
    for (const [sq, piece] of board) {
      const g = this.makePiece(piece.kind, piece.color);
      g.position.copy(squarePos(sq));
      this.pieceLayer.add(g);
      this.pieces.set(sq, g);
    }
  }

  // ------------------------------------------------------------------ coups

  private tween(duration: number, update: (k: number) => void): Promise<void> {
    return new Promise((resolve) => {
      this.tweens.push({ start: performance.now(), duration, update, done: resolve });
    });
  }

  private slide(g: THREE.Group, to: string, lift: number): Promise<void> {
    const from = g.position.clone();
    const target = squarePos(to);
    return this.tween(MOVE_DURATION, (k) => {
      const e = ease(k);
      g.position.lerpVectors(from, target, e);
      g.position.y = Math.sin(Math.PI * e) * lift;
    });
  }

  private vanish(g: THREE.Group): Promise<void> {
    return this.tween(380, (k) => {
      g.scale.setScalar(Math.max(0.001, 1 - k));
      g.position.y = -0.4 * k;
    }).then(() => {
      this.pieceLayer.remove(g);
    });
  }

  /** Joue un coup UCI (« e2e4 », « e7e8q ») avec animation. */
  async move(uci: string) {
    const from = uci.slice(0, 2);
    const to = uci.slice(2, 4);
    const promo = uci[4] as PieceKind | undefined;
    const g = this.pieces.get(from);
    if (!g) return;
    const kind = g.userData.kind as PieceKind;
    const jobs: Promise<void>[] = [];

    // Prise (y compris en passant : pion qui change de colonne vers une case vide).
    let capturedSq: string | null = this.pieces.has(to) ? to : null;
    if (!capturedSq && kind === "p" && from[0] !== to[0]) capturedSq = to[0] + from[1];
    if (capturedSq) {
      const victim = this.pieces.get(capturedSq)!;
      this.pieces.delete(capturedSq);
      jobs.push(new Promise((r) => setTimeout(r, MOVE_DURATION * 0.55)).then(() => this.vanish(victim)));
    }

    this.pieces.delete(from);
    this.pieces.set(to, g);
    const dist = squarePos(from).distanceTo(squarePos(to));
    jobs.push(this.slide(g, to, kind === "n" ? 0.9 : 0.25 + dist * 0.06));

    // Roque : la tour suit le roi.
    if (kind === "k" && Math.abs(FILES.indexOf(from[0] as never) - FILES.indexOf(to[0] as never)) === 2) {
      const kingSide = to[0] === "g";
      const rFrom = (kingSide ? "h" : "a") + from[1];
      const rTo = (kingSide ? "f" : "d") + from[1];
      const rook = this.pieces.get(rFrom);
      if (rook) {
        this.pieces.delete(rFrom);
        this.pieces.set(rTo, rook);
        jobs.push(this.slide(rook, rTo, 0.6));
      }
    }
    await Promise.all(jobs);

    // Promotion : le pion devient la pièce choisie.
    if (promo && kind === "p") {
      const color = g.userData.color as "w" | "b";
      this.pieceLayer.remove(g);
      const promoted = this.makePiece(promo, color);
      promoted.position.copy(squarePos(to));
      this.pieceLayer.add(promoted);
      this.pieces.set(to, promoted);
      await this.tween(300, (k) => promoted.scale.setScalar(0.4 + 0.6 * ease(k)));
    }
  }

  /** Toutes les pièces s'effacent (fin de partie, avant la suivante). */
  async clear() {
    const all = [...this.pieces.values()];
    this.pieces.clear();
    await Promise.all(all.map((g) => this.vanish(g)));
  }

  // ------------------------------------------------------------------ orientation

  private applyOrientation() {
    const q = this.userQ.clone();
    q.multiply(new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), BASE_TILT + this.roll));
    q.multiply(new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), this.spin));
    this.root.quaternion.copy(q);
  }

  private onDown = (e: PointerEvent) => {
    this.dragging = true;
    this.lastPointer = { x: e.clientX, y: e.clientY };
    this.canvas.setPointerCapture(e.pointerId);
  };

  private onMove = (e: PointerEvent) => {
    if (!this.dragging) return;
    const dx = e.clientX - this.lastPointer.x;
    const dy = e.clientY - this.lastPointer.y;
    this.lastPointer = { x: e.clientX, y: e.clientY };
    // Rotation libre dans tous les sens, autour des axes de l'écran.
    const qy = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), dx * 0.009);
    const qx = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), dy * 0.009);
    this.userQ.premultiply(qy).premultiply(qx);
    this.applyOrientation();
  };

  private onUp = (e: PointerEvent) => {
    this.dragging = false;
    this.idleSince = performance.now();
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
  };

  // ------------------------------------------------------------------ boucle

  resize(width: number, height: number) {
    if (!width || !height) return;
    this.renderer.setSize(width, height, false);
    this.camera.aspect = width / height;
    // Le plateau (et ses pièces) tient dans le cadre quel que soit l'angle.
    const radius = 6.6;
    const vFov = (this.camera.fov * Math.PI) / 180;
    const hFov = 2 * Math.atan(Math.tan(vFov / 2) * this.camera.aspect);
    const dist = radius / Math.sin(Math.min(vFov, hFov) / 2);
    this.camera.position.set(0, 0, dist);
    this.camera.updateProjectionMatrix();
    this.render();
  }

  start() {
    if (this.running) return;
    this.running = true;
    this.last = performance.now();
    if (!this.nextRoll) this.nextRoll = this.last + ROLL_EVERY / 2;
    const loop = (t: number) => {
      if (!this.running) return;
      this.tick(t);
      this.frame = requestAnimationFrame(loop);
    };
    this.frame = requestAnimationFrame(loop);
  }

  stop() {
    this.running = false;
    cancelAnimationFrame(this.frame);
  }

  private tick(now: number) {
    const dt = Math.min(0.1, (now - this.last) / 1000);
    this.last = now;
    if (!this.dragging) {
      this.spin += dt * SPIN_SPEED * (this.reducedMotion ? 0.35 : 1);
      // Retour en douceur vers l'orientation de repos après une manipulation.
      if (now - this.idleSince > 2500) this.userQ.slerp(new THREE.Quaternion(), Math.min(1, dt * 1.4));
      // Retournement complet, de temps en temps.
      if (!this.reducedMotion) {
        if (!this.rollStart && now >= this.nextRoll) this.rollStart = now;
        if (this.rollStart) {
          const k = Math.min(1, (now - this.rollStart) / ROLL_DURATION);
          this.roll = ease(k) * Math.PI * 2;
          if (k >= 1) {
            this.roll = 0;
            this.rollStart = 0;
            this.nextRoll = now + ROLL_EVERY;
          }
        }
      }
      this.applyOrientation();
    }
    this.tweens = this.tweens.filter((tw) => {
      const k = Math.min(1, (now - tw.start) / tw.duration);
      tw.update(k);
      if (k >= 1) tw.done();
      return k < 1;
    });
    this.render();
  }

  private render() {
    this.renderer.render(this.scene, this.camera);
  }

  dispose() {
    this.stop();
    this.canvas.removeEventListener("pointerdown", this.onDown);
    this.canvas.removeEventListener("pointermove", this.onMove);
    this.canvas.removeEventListener("pointerup", this.onUp);
    this.canvas.removeEventListener("pointercancel", this.onUp);
    this.tweens.forEach((t) => t.done());
    this.tweens = [];
    this.scene.traverse((o) => {
      if (o instanceof THREE.Mesh && !(o.parent && o.parent.userData.kind)) o.geometry.dispose();
    });
    this.boardMat?.squares.dispose();
    this.boardMat?.frame.dispose();
    this.pieceMat?.white.dispose();
    this.pieceMat?.black.dispose();
    this.scene.environment?.dispose();
    this.renderer.dispose();
  }
}
