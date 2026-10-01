// Matières des pièces (selon le jeu de pièces choisi dans Apparence) et textures du plateau
// (selon le plateau choisi), dessinées une fois dans un canevas : veinage du bois, veines du
// marbre, trame Persona, papier et lavis d'encre.

import * as THREE from "three";
import { BOARDS, type BoardId, type PieceSetId } from "@/lib/themes";

export interface PieceMaterials {
  white: THREE.Material;
  black: THREE.Material;
}

export function pieceMaterials(set: PieceSetId): PieceMaterials {
  switch (set) {
    case "crystal":
      return {
        white: new THREE.MeshPhysicalMaterial({
          color: "#ffffff",
          transmission: 1,
          thickness: 0.6,
          roughness: 0.04,
          ior: 1.52,
          envMapIntensity: 1.3,
          clearcoat: 1,
        }),
        black: new THREE.MeshPhysicalMaterial({
          color: "#4a5170",
          transmission: 0.85,
          thickness: 0.8,
          roughness: 0.06,
          ior: 1.52,
          attenuationColor: new THREE.Color("#141826"),
          attenuationDistance: 0.6,
          envMapIntensity: 1.3,
          clearcoat: 1,
        }),
      };
    case "holo":
      return {
        white: new THREE.MeshPhysicalMaterial({
          color: "#e6f6ff",
          metalness: 0.25,
          roughness: 0.12,
          iridescence: 1,
          iridescenceIOR: 1.6,
          clearcoat: 1,
        }),
        black: new THREE.MeshPhysicalMaterial({
          color: "#2c1950",
          metalness: 0.35,
          roughness: 0.14,
          iridescence: 1,
          iridescenceIOR: 1.8,
          clearcoat: 1,
        }),
      };
    case "persona":
      return {
        white: new THREE.MeshToonMaterial({ color: "#f6f6f6" }),
        black: new THREE.MeshToonMaterial({ color: "#151515", emissive: "#3a0008" }),
      };
    case "ink":
      return {
        white: new THREE.MeshStandardMaterial({ color: "#efe6d2", roughness: 0.92 }),
        black: new THREE.MeshStandardMaterial({ color: "#24211e", roughness: 0.85 }),
      };
    default:
      // Staunton : buis ciré et ébène vernie.
      return {
        white: new THREE.MeshPhysicalMaterial({
          color: "#ead8b2",
          roughness: 0.42,
          clearcoat: 0.6,
          clearcoatRoughness: 0.3,
        }),
        black: new THREE.MeshPhysicalMaterial({
          color: "#2a1c15",
          roughness: 0.32,
          clearcoat: 0.85,
          clearcoatRoughness: 0.2,
        }),
      };
  }
}

/** Couleur du cadre et du dessous du plateau. */
const FRAME: Record<BoardId, string> = {
  wood: "#3a2417",
  marble: "#29292e",
  crystal: "#8fa6bb",
  persona5: "#111111",
  persona3: "#0b1f4d",
  sumi: "#2e2a26",
};

// Petit générateur pseudo-aléatoire : le même plateau à chaque ouverture.
function random(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

function squaresTexture(board: BoardId): THREE.CanvasTexture {
  const size = 1024;
  const cell = size / 8;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = size;
  const g = canvas.getContext("2d")!;
  const info = BOARDS.find((b) => b.id === board) ?? BOARDS[0];
  const rand = random(7 + board.length * 31);
  for (let r = 0; r < 8; r++) {
    for (let f = 0; f < 8; f++) {
      const light = (r + f) % 2 === 0;
      const x = f * cell;
      const y = r * cell;
      g.fillStyle = light ? info.light : info.dark;
      g.fillRect(x, y, cell, cell);
      g.save();
      g.beginPath();
      g.rect(x, y, cell, cell);
      g.clip();
      if (board === "wood") {
        // Veinage, orienté différemment d'une case à l'autre.
        const vertical = (r + f) % 2 === 0;
        g.strokeStyle = light ? "rgba(120, 80, 40, 0.16)" : "rgba(255, 220, 180, 0.07)";
        for (let i = 0; i < 22; i++) {
          g.lineWidth = 1 + rand() * 2.5;
          g.beginPath();
          const off = rand() * cell;
          for (let t = 0; t <= cell; t += 8) {
            const w = Math.sin(t / 18 + i) * 3 + Math.sin(t / 47 + off) * 4;
            if (vertical) g.lineTo(x + off + w, y + t);
            else g.lineTo(x + t, y + off + w);
          }
          g.stroke();
        }
      } else if (board === "marble" || board === "sumi") {
        // Veines du marbre ; taches de lavis pour l'encre.
        g.strokeStyle = light ? "rgba(90, 90, 100, 0.18)" : "rgba(220, 220, 230, 0.12)";
        for (let i = 0; i < 4; i++) {
          g.lineWidth = 0.6 + rand() * 1.4;
          g.beginPath();
          let px = x + rand() * cell;
          let py = y;
          g.moveTo(px, py);
          while (py < y + cell) {
            px += (rand() - 0.5) * 22;
            py += 6 + rand() * 10;
            g.lineTo(px, py);
          }
          g.stroke();
        }
        if (board === "sumi" && !light) {
          g.fillStyle = "rgba(0, 0, 0, 0.18)";
          g.beginPath();
          g.arc(x + rand() * cell, y + rand() * cell, cell * (0.2 + rand() * 0.3), 0, Math.PI * 2);
          g.fill();
        }
      } else if (board === "persona5" && !light) {
        // Trame de points façon bande dessinée.
        g.fillStyle = "rgba(0, 0, 0, 0.22)";
        for (let py = 6; py < cell; py += 12)
          for (let px = (py / 12) % 2 ? 12 : 6; px < cell; px += 12) {
            g.beginPath();
            g.arc(x + px, y + py, 2.6, 0, Math.PI * 2);
            g.fill();
          }
      } else if (board === "persona3") {
        g.strokeStyle = light ? "rgba(30, 80, 200, 0.12)" : "rgba(160, 210, 255, 0.12)";
        g.lineWidth = 2;
        for (let i = -cell; i < cell; i += 14) {
          g.beginPath();
          g.moveTo(x + i, y);
          g.lineTo(x + i + cell, y + cell);
          g.stroke();
        }
      } else if (board === "crystal") {
        const grad = g.createLinearGradient(x, y, x + cell, y + cell);
        grad.addColorStop(0, "rgba(255, 255, 255, 0.18)");
        grad.addColorStop(1, "rgba(255, 255, 255, 0)");
        g.fillStyle = grad;
        g.fillRect(x, y, cell, cell);
      }
      g.restore();
    }
  }
  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = 8;
  return texture;
}

export interface BoardMaterials {
  squares: THREE.Material;
  frame: THREE.Material;
}

export function boardMaterials(board: BoardId): BoardMaterials {
  const map = squaresTexture(board);
  const glassy = board === "crystal";
  return {
    squares: new THREE.MeshPhysicalMaterial({
      map,
      roughness: glassy ? 0.08 : board === "marble" ? 0.25 : 0.55,
      clearcoat: glassy || board === "marble" ? 1 : 0.35,
      clearcoatRoughness: 0.15,
    }),
    frame: new THREE.MeshPhysicalMaterial({
      color: FRAME[board],
      roughness: glassy ? 0.1 : 0.45,
      metalness: glassy ? 0.1 : 0,
      clearcoat: 0.6,
    }),
  };
}
