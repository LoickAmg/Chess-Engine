// Petits sons synthétisés (Web Audio) : aucun fichier audio à embarquer.

let ctx: AudioContext | null = null;
let enabled = true;

export function setSoundEnabled(on: boolean) {
  enabled = on;
}

function audio(): AudioContext | null {
  if (!enabled) return null;
  try {
    ctx ??= new AudioContext();
    if (ctx.state === "suspended") void ctx.resume();
    return ctx;
  } catch {
    return null;
  }
}

function tone(freq: number, start: number, dur: number, type: OscillatorType = "sine", gain = 0.12) {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + start;
  const osc = a.createOscillator();
  const g = a.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t0);
  g.gain.setValueAtTime(0, t0);
  g.gain.linearRampToValueAtTime(gain, t0 + 0.008);
  g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
  osc.connect(g).connect(a.destination);
  osc.start(t0);
  osc.stop(t0 + dur + 0.02);
}

/** Bruit bref filtré : le « toc » d'une pièce posée sur le plateau. */
function knock(start: number, freq: number, gain: number) {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + start;
  const len = Math.floor(a.sampleRate * 0.06);
  const buffer = a.createBuffer(1, len, a.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < len; i++) data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 3);
  const src = a.createBufferSource();
  src.buffer = buffer;
  const filter = a.createBiquadFilter();
  filter.type = "bandpass";
  filter.frequency.value = freq;
  filter.Q.value = 1.4;
  const g = a.createGain();
  g.gain.value = gain;
  src.connect(filter).connect(g).connect(a.destination);
  src.start(t0);
}

export const sfx = {
  move: () => knock(0, 900, 0.9),
  capture: () => {
    knock(0, 700, 1.1);
    knock(0.05, 1300, 0.6);
  },
  check: () => {
    knock(0, 900, 0.8);
    tone(880, 0.02, 0.18, "triangle", 0.06);
  },
  select: () => tone(1200, 0, 0.05, "sine", 0.03),
  star: () => {
    tone(1318, 0, 0.12, "triangle", 0.08);
    tone(1760, 0.07, 0.18, "triangle", 0.07);
  },
  wrong: () => {
    tone(220, 0, 0.16, "square", 0.04);
    tone(180, 0.09, 0.2, "square", 0.04);
  },
  success: () => [523, 659, 784, 1047].forEach((f, i) => tone(f, i * 0.08, 0.3, "triangle", 0.08)),
  fanfare: () =>
    [523, 659, 784, 1047, 784, 1047, 1319].forEach((f, i) => tone(f, i * 0.09, 0.35, "triangle", 0.08)),
};
