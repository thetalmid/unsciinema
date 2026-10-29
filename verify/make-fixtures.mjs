/** Generates the non-PNG fixtures using real engines:
 *  - animated GIF: our own wasm GifWriter (2 visibly different frames,
 *    unequal delays, transparency on frame 2, loop forever);
 *  - JPEG/WebP: headless Chromium canvas encoders (real browser output). */
import fs from "node:fs";
import { chromium } from "playwright";
import * as wasm from "../frontend/src/generated/wasm/unsciinema_core.js";

const D = new URL("../test-fixtures/", import.meta.url);
const out = (n) => new URL(n, D).pathname;

// --- animated GIF via wasm ---
const bytes = fs.readFileSync(
  new URL("../frontend/src/generated/wasm/unsciinema_core_bg.wasm", import.meta.url),
);
await wasm.default({ module_or_path: bytes });

const W = 48, H = 36;
const f1 = new Uint8Array(W * H * 4);
const f2 = new Uint8Array(W * H * 4);
for (let y = 0; y < H; y++) {
  for (let x = 0; x < W; x++) {
    const i = (y * W + x) * 4;
    // frame 1: red/blue diagonal
    f1[i] = (x * 8) & 255; f1[i + 1] = 40; f1[i + 2] = (y * 8) & 255; f1[i + 3] = 255;
    // frame 2: left half FULLY transparent (whole cells at any cell size ->
    // survives conversion as transparent cells), right half green field.
    f2[i] = 20; f2[i + 1] = (x * 6) & 255; f2[i + 2] = 60; f2[i + 3] = x < 24 ? 0 : 255;
  }
}
const gw = new wasm.GifWriter(W, H, 0); // loop forever
gw.addFrame(f1, 400);                    // unequal delays
gw.addFrame(f2, 150);
let gif = gw.finish();

// Disposal change on input: our writer emits disposal=Background (2) for all
// frames. Frame 1 MUST stay Background so frame 2's transparent left half
// composites over a cleared canvas (real transparency downstream). Rewrite
// FRAME 2's disposal to Keep (1) instead: still a disposal change across
// frames (mission: "a disposal change") without destroying transparency.
{
  const b = Buffer.from(gif);
  let p = 13;
  const packed = b[10];
  if (packed & 0x80) p += 3 * (2 << (packed & 7));
  let patched = false;
  while (p < b.length) {
    if (b[p] === 0x21 && b[p + 1] === 0xf9) {
      // GCE: 21 F9 04 <flags> <delayL> <delayH> <tIdx> 00
      // Skip frame 1 (keep Background=2); patch frame 2 to Keep(1)+transparency.
      if (b[p + 3] === 9) {
        b[p + 3] = 5; // disposal Keep(1)<<2 | transparency bit
        patched = true;
        break;
      }
      p += 2;
      while (b[p] !== 0) p += b[p] + 1;
      p++;
      continue;
    }
    if (b[p] === 0x2c) {
      p += 10;
      if (b[p - 1] & 0x80) p += 3 * (2 << (b[p - 1] & 7));
      p++; // LZW min code size
      while (b[p] !== 0) p += b[p] + 1;
      p++;
    } else if (b[p] === 0x21) {
      p += 2;
      while (b[p] !== 0) p += b[p] + 1;
      p++;
    } else if (b[p] === 0x3b) break;
    else throw new Error("GIF walk failed at " + p);
  }
  if (!patched) throw new Error("frame1 GCE not found");
  gif = b;
}
fs.writeFileSync(out("anim-2frame-unequal-delays.gif"), gif);
console.log("gif bytes", gif.length, "frame1 GCE flags=8 (Background), frame2 flags=5 (Keep+transparent)");

// --- JPEG + WebP via headless chromium ---
const browser = await chromium.launch();
const page = await browser.newPage();
await page.goto("about:blank");
const jpg = await page.evaluate(() => {
  const c = document.createElement("canvas");
  c.width = 64; c.height = 48;
  const g = c.getContext("2d");
  const grad = g.createLinearGradient(0, 0, 64, 48);
  grad.addColorStop(0, "#ff4400"); grad.addColorStop(0.5, "#0044ff"); grad.addColorStop(1, "#00ff66");
  g.fillStyle = grad; g.fillRect(0, 0, 64, 48);
  g.fillStyle = "#ffffff"; g.fillRect(8, 8, 16, 12);
  g.fillStyle = "#000000"; g.fillRect(40, 24, 16, 12);
  const blobB64 = (type) => new Promise((res) =>
    c.toBlob((b) => { const r = new FileReader(); r.onload = () => res(r.result.split(",")[1]); r.readAsDataURL(b); }, type, 0.92));
  return blobB64("image/jpeg");
});
fs.writeFileSync(out("photo-64x48.jpg"), Buffer.from(jpg, "base64"));
const webp = await page.evaluate(() => {
  const c = document.createElement("canvas");
  c.width = 40; c.height = 40;
  const g = c.getContext("2d");
  g.fillStyle = "#336699"; g.fillRect(0, 0, 40, 40);
  g.fillStyle = "#ffcc00"; g.beginPath(); g.arc(20, 20, 12, 0, 7); g.fill();
  return new Promise((res) =>
    c.toBlob((b) => { const r = new FileReader(); r.onload = () => res(r.result.split(",")[1]); r.readAsDataURL(b); }, "image/webp"));
});
fs.writeFileSync(out("photo-40x40.webp"), Buffer.from(webp, "base64"));
await browser.close();

for (const f of fs.readdirSync(D)) {
  console.log(f, fs.statSync(new URL(f, D)).size);
}
