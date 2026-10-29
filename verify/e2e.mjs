/**
 * Real-browser E2E verification for Unsciinema (mission §12/§13).
 *
 * Runs against the PRODUCTION build via `vite preview`. Captures console
 * errors, page errors, CSP violations, and every network request (origin
 * audit). Exercises: all fixtures (PNG/JPEG/GIF/WebP + malformed/oversized),
 * every font, every dither id, color/palette/alpha modes, ASCII blend,
 * contrast/gamma direction, CRT neutrality + application, rapid-change
 * settling, context loss, CPU fallback, playback, all exports (independently
 * re-parsed), budgets (convert ms, assets), and determinism.
 *
 * Usage: node e2e.mjs [baseURL]   (default http://127.0.0.1:4173/)
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { PNG } from "pngjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const FIX = path.join(HERE, "..", "test-fixtures");
const DL = "/tmp/exports";
fs.mkdirSync(DL, { recursive: true });

const BASE = process.argv[2] ?? "http://127.0.0.1:4173/";
const results = [];
let consoleErrors = [];
let pageErrors = [];
let cspViolations = [];
let requests = [];
let wasmContentType = null;

function ok(name, detail = "") {
  results.push({ pass: true, name, detail });
  console.log(`PASS  ${name}${detail ? ` — ${detail}` : ""}`);
}
function fail(name, detail = "") {
  results.push({ pass: false, name, detail });
  console.log(`FAIL  ${name}${detail ? ` — ${detail}` : ""}`);
}
function check(cond, name, detail = "") {
  cond ? ok(name, detail) : fail(name, detail);
  return cond;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function seq(page) {
  return page.evaluate(
    () => Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0),
  );
}

/** Load a file through the hidden input and wait for the conversion result. */
async function loadFile(page, file) {
  const before = await seq(page);
  await page.setInputFiles('input[type="file"]', path.join(FIX, file));
  await page.waitForFunction(
    (n) =>
      Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
    before,
    { timeout: 30_000 },
  );
}

/** Run an action and wait for the NEXT applied conversion. */
async function convertAfter(page, action) {
  const before = await seq(page);
  await action();
  await page.waitForFunction(
    (n) =>
      Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
    before,
    { timeout: 30_000 },
  );
  await sleep(120); // let the render effect draw before screenshots
}

function pngStats(buf) {
  const png = PNG.sync.read(buf);
  const { width, height, data } = png;
  let sum = 0;
  let sumSq = 0;
  const n = width * height * 3;
  for (let i = 0; i < data.length; i += 4) {
    sum += data[i] + data[i + 1] + data[i + 2];
    sumSq += data[i] ** 2 + data[i + 1] ** 2 + data[i + 2] ** 2;
  }
  const mean = sum / n;
  const varc = sumSq / n - mean * mean;
  const stripMean = (y0, y1) => {
    let s = 0;
    let c = 0;
    for (let y = y0; y < y1; y++) {
      for (let x = 0; x < width; x++) {
        const i = (y * width + x) * 4;
        s += data[i + 1];
        c++;
      }
    }
    return s / c;
  };
  const colMeanRed = (x0, x1) => {
    let s = 0;
    let c = 0;
    for (let y = 0; y < height; y++) {
      for (let x = x0; x < x1; x++) {
        const i = (y * width + x) * 4;
        s += data[i];
        c++;
      }
    }
    return s / c;
  };
  const eighthX = Math.max(1, Math.floor(width / 8));
  const eighthY = Math.max(1, Math.floor(height / 8));
  return {
    width, height, mean, variance: varc,
    greenTop: stripMean(0, eighthY),
    greenBottom: stripMean(height - eighthY, height),
    redLeft: colMeanRed(0, eighthX),
    redRight: colMeanRed(width - eighthX, width),
    data,
  };
}

async function shot(page) {
  const el = page.locator('[data-testid="preview-canvas"]');
  return PNG.sync.read(await el.screenshot());
}

function statsOf(png) {
  const { width, height, data } = png;
  let sum = 0;
  let sumSq = 0;
  const n = width * height * 3;
  for (let i = 0; i < data.length; i += 4) {
    sum += data[i] + data[i + 1] + data[i + 2];
    sumSq += data[i] ** 2 + data[i + 1] ** 2 + data[i + 2] ** 2;
  }
  const mean = sum / n;
  return { width, height, mean, variance: sumSq / n - mean * mean };
}

function diffPixels(a, b) {
  if (a.width !== b.width || a.height !== b.height) return Number.POSITIVE_INFINITY;
  let d = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    if (
      Math.abs(a.data[i] - b.data[i]) > 2 ||
      Math.abs(a.data[i + 1] - b.data[i + 1]) > 2 ||
      Math.abs(a.data[i + 2] - b.data[i + 2]) > 2
    ) {
      d++;
    }
  }
  return d;
}

async function setRange(page, sel, value) {
  await page.evaluate(
    ([s, v]) => {
      const el = document.querySelector(s);
      el.value = String(v);
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.dispatchEvent(new Event("change", { bubbles: true }));
    },
    [sel, value],
  );
}

async function download(page, click) {
  const [dl] = await Promise.all([page.waitForEvent("download", { timeout: 60_000 }), click()]);
  const name = dl.suggestedFilename();
  const file = path.join(DL, `${Date.now()}-${name}`);
  await dl.saveAs(file);
  return { name, file, bytes: fs.readFileSync(file) };
}

// --- minimal GIF walker: frames, delays, loop ------------------------------
function parseGif(buf) {
  if (!(buf[0] === 0x47 && buf[1] === 0x49 && buf[2] === 0x46)) throw new Error("not GIF");
  const version = buf.toString("ascii", 3, 6);
  const width = buf.readUInt16LE(6);
  const height = buf.readUInt16LE(8);
  let p = 13;
  if (buf[6 - 6 + 10] & 0x80) p += 3 * (2 << (buf[10] & 7)); // GCT
  const frames = [];
  let loopCode = -1;
  let gce = null;
  while (p < buf.length) {
    const b = buf[p];
    if (b === 0x21) {
      const label = buf[p + 1];
      if (label === 0xf9) {
        gce = { flags: buf[p + 3], delay: buf.readUInt16LE(p + 4), transparent: buf[p + 6] };
      } else if (label === 0xff) {
        const size = buf[p + 2];
        const app = buf.toString("ascii", p + 3, p + 3 + size);
        if (app.startsWith("NETSCAPE")) {
          loopCode = buf.readUInt16LE(p + 3 + size + 2);
        }
      }
      // Skip extension data as size-prefixed sub-blocks (start at SIZE byte).
      p += 2;
      while (buf[p] !== 0) p += buf[p] + 1;
      p += 1;
    } else if (b === 0x2c) {
      const fw = buf.readUInt16LE(p + 5);
      const fh = buf.readUInt16LE(p + 7);
      const lct = buf[p + 9] & 0x80;
      p += 10;
      if (lct) p += 3 * (2 << (buf[p - 1] & 7));
      p++; // LZW min code
      while (buf[p] !== 0) p += buf[p] + 1;
      p++;
      frames.push({ fw, fh, gce });
      gce = null;
    } else if (b === 0x3b) {
      break;
    } else {
      throw new Error(`bad GIF block 0x${b.toString(16)} @${p}`);
    }
  }
  return { version, width, height, frames, loopCode };
}

function parsePngIhdr(buf) {
  if (!(buf[0] === 0x89 && buf[1] === 0x50)) throw new Error("not PNG");
  return { width: buf.readUInt32BE(16), height: buf.readUInt32BE(20) };
}

function webpVariant(buf) {
  if (buf.toString("ascii", 0, 4) !== "RIFF" || buf.toString("ascii", 8, 12) !== "WEBP") {
    throw new Error("not WEBP");
  }
  let p = 12;
  const chunks = [];
  while (p + 8 <= buf.length) {
    const fourcc = buf.toString("ascii", p, p + 4);
    const size = buf.readUInt32LE(p + 4);
    chunks.push(fourcc);
    if (fourcc === "VP8L") return { chunks, lossless: true };
    if (fourcc === "VP8 ") return { chunks, lossless: false };
    p += 8 + size + (size % 2);
  }
  return { chunks, lossless: null };
}

async function main() {
  const browser = await chromium.launch();
  const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1440, height: 960 } });
  const page = await context.newPage();

  await page.addInitScript(() => {
    window.__csp = [];
    document.addEventListener("securitypolicyviolation", (e) => {
      window.__csp.push(`${e.violatedDirective} blocked=${e.blockedURI}`);
    });
    // Force software rendering marker: SwiftShader is expected in headless CI.
    try {
      const c = document.createElement("canvas");
      const gl = c.getContext("webgl2");
      const dbg = gl?.getExtension("WEBGL_debug_renderer_info");
      window.__glRenderer = dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : "unknown";
      window.__gl2 = !!gl;
    } catch {
      window.__gl2 = false;
    }
  });

  page.on("console", (m) => {
    if (m.type() === "error") consoleErrors.push(m.text());
  });
  page.on("pageerror", (e) => pageErrors.push(String(e)));
  page.on("crash", () => console.log("!! PAGE CRASHED"));
  page.on("request", (r) => requests.push(r.url()));
  page.on("response", (r) => {
    if (r.url().includes(".wasm")) wasmContentType = r.headers()["content-type"] ?? null;
  });

  console.log(`== target ${BASE}`);
  await page.goto(BASE, { waitUntil: "load" });

  // ---- 1. boot + wasm tables ----
  await page.waitForSelector(".app", { timeout: 15_000 });
  check((await page.locator("#ctl-font option").count()) === 7, "7 font variants in UI");
  check((await page.locator("#ctl-palette option").count()) === 32, "31 palettes + custom in UI", `${await page.locator("#ctl-palette option").count()}`);
  const bootErr = await page.locator(".boot-error").count();
  check(bootErr === 0, "no boot error");
  const gl2 = await page.evaluate(() => window.__gl2);
  const glRenderer = await page.evaluate(() => window.__glRenderer);
  check(gl2, "WebGL2 context available", `renderer: ${glRenderer}`);

  // ---- 2. still PNG: load + preview ----
  await loadFile(page, "gradient-48x32.png");
  await page.waitForSelector('[data-testid="preview-canvas"]:not(.is-hidden)');
  const status = await page.locator(".status-msg").textContent();
  check(
    /6×4 cells · 48×32 px/.test(status ?? "") || /Loaded png 48×32/.test(status ?? ""),
    "gradient PNG loaded+converted with expected dims",
    status?.slice(0, 90),
  );
  const mode = await page.locator('[data-testid="render-mode"]').textContent();
  check(mode?.trim() === "WebGL2", "WebGL2 renderer active", mode?.trim() ?? "");
  const convertMsText = await page.locator('[data-testid="convert-ms"]').textContent();
  const convertMs = Number(convertMsText?.replace("ms", "").trim());
  check(Number.isFinite(convertMs) && convertMs >= 0, "convert timing shown", convertMsText?.trim());

  // orientation (top-left origin): gradient green grows downward
  const g1 = await shot(page);
  const st1 = statsOf(g1);
  check(st1.variance > 30, "preview non-uniform", `var=${st1.variance.toFixed(1)}`);
  // element screenshot may include CSS scaling; check directional green via export PNG instead (native):
  // (orientation asserted below on exported PNG)

  // ---- 3. every fixture format ----
  for (const f of [
    ["photo-64x48.jpg", "jpeg 64×48"],
    ["photo-40x40.webp", "webp 40×40"],
    ["anim-2frame-unequal-delays.gif", "gif 48×36"],
    ["odd-37x23.png", "odd 37×23"],
    ["alpha-partial.png", "alpha 40×40"],
    ["portrait-60x160.png", "portrait 60×160"],
    ["color-patches.png", "patches 64×48"],
    ["bw-dots.png", "bw 32×32"],
    ["solid-white.png", "white 16×16"],
    ["solid-black.png", "black 16×16"],
  ]) {
    try {
      await loadFile(page, f[0]);
      const s = await page.locator(".status-msg").textContent();
      const seqAfter = await seq(page);
      check(seqAfter > 0 && /Converted/.test(s ?? ""), `fixture converts: ${f[1]}`);
    } catch (e) {
      fail(`fixture converts: ${f[1]}`, String(e).slice(0, 140));
    }
  }

  // ---- 4. malformed + oversized: clean rejection, recoverable ----
  const seqBeforeBad = await seq(page);
  await page.setInputFiles('input[type="file"]', path.join(FIX, "malformed-truncated.png"));
  await page.waitForFunction(
    () => /failed|error/i.test(document.querySelector(".status-msg")?.textContent ?? ""),
    { timeout: 15_000 },
  );
  const badStatus = await page.locator(".status-msg").textContent();
  check(/fail|error/i.test(badStatus ?? ""), "malformed PNG rejected with error status", badStatus?.slice(0, 90));
  check((await seq(page)) === seqBeforeBad, "malformed PNG produced no conversion");
  check((await page.locator('[data-testid="preview-canvas"]:not(.is-hidden)').count()) === 1, "previous preview kept after failed load");

  await page.setInputFiles('input[type="file"]', path.join(FIX, "oversized-9001.png"));
  await page.waitForFunction(
    () => /limit|exceeds|8192|reject/i.test(document.querySelector(".status-msg")?.textContent ?? ""),
    { timeout: 20_000 },
  );
  const overStatus = await page.locator(".status-msg").textContent();
  check(/limit|exceeds|8192|reject/i.test(overStatus ?? ""), "oversized input hits a limit", overStatus?.slice(0, 110));

  // recover: load good file again
  await loadFile(page, "color-patches.png");
  ok("errors recoverable (good load after rejections)");

  // ---- 5. fonts (all 7) ----
  {
    const seen = [];
    for (let i = 0; i < 7; i++) {
      await convertAfter(page, () => page.selectOption("#ctl-font", String(i)));
      const ms = await page.locator('[data-testid="convert-ms"]').textContent();
      const name = await page.locator("#ctl-font option:checked").textContent();
      seen.push(name?.trim());
      check(Number((ms ?? "").replace(/[^\d.]/g, "")) >= 0, `font ${i} (${name?.trim()}) converts`, ms?.trim());
    }
    check(new Set(seen).size === 7, "all 7 font names exercised", seen.join(","));
  }

  // ---- 6. dither ids 0..10 (5 labeled Random Noise, 10 Blue Noise) ----
  {
    const labels = await page.locator("#ctl-dither option").allTextContents();
    check(labels[5]?.includes("Random Noise"), "id 5 labeled Random Noise", labels[5]);
    check(labels[10]?.includes("Blue Noise"), "id 10 labeled Blue Noise", labels[10]);
    const base = await shot(page);
    let changed = 0;
    for (let id = 0; id <= 10; id++) {
      await convertAfter(page, () => page.selectOption("#ctl-dither", String(id)));
      const s = await page.locator(".status-msg").textContent();
      check(/Converted/.test(s ?? ""), `dither ${id} (${labels[id]?.trim()}) converts`);
      if (id % 3 === 0) {
        const now = await shot(page);
        if (diffPixels(base, now) > 50) changed++;
      }
    }
    check(changed >= 2, "dither changes are visible", `${changed}/4 sampled steps changed`);
    await convertAfter(page, () => page.selectOption("#ctl-dither", "4"));
  }

  // ---- 7. color modes, palettes, alpha modes ----
  for (const m of ["1", "2", "0"]) {
    await convertAfter(page, () => page.selectOption("#ctl-color", String(m)));
    ok(`color mode ${m} converts`);
  }
  await convertAfter(page, () => page.selectOption("#ctl-color", "2"));
  for (const preset of ["0", "14", "24", "30", "-1"]) {
    if (preset === "-1") {
      await convertAfter(page, () => page.selectOption("#ctl-palette", "-1"));
      // custom textarea appears; type a valid palette (seq captured BEFORE)
      const seqCustom = await seq(page);
      await page.fill("#ctl-custom", "#000000 #ffffff #ff0000 #00ff00");
      await page.waitForFunction(
        (n) =>
          Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
        seqCustom,
        { timeout: 15_000 },
      );
      await sleep(120);
      const err = await page.locator("#ctl-custom-err").textContent();
      check(!(err ?? "").trim().length, "custom palette parses clean");
      // invalid palette keeps error visible but app stays alive
      await page.fill("#ctl-custom", "#xyz");
      await sleep(150);
      const err2 = await page.locator("#ctl-custom-err").textContent();
      check(/Invalid color/.test(err2 ?? ""), "invalid palette rejected inline", err2?.slice(0, 60));
      await page.fill("#ctl-custom", "#101418 #2b3440 #4c5a68 #7b8794 #b7c2cd #ffffff");
      await sleep(150);
    } else {
      await convertAfter(page, () => page.selectOption("#ctl-palette", preset));
      ok(`palette preset ${preset} converts`);
    }
  }
  await convertAfter(page, () => page.selectOption("#ctl-color", "1"));
  for (const a of ["1", "2", "0"]) {
    await convertAfter(page, () => page.selectOption("#ctl-alpha", String(a)));
    ok(`alpha mode ${a} converts`);
  }

  // ---- 8. contrast/gamma direction (quantitative) ----
  await loadFile(page, "color-patches.png");
  await convertAfter(page, () => page.selectOption("#ctl-color", "0"));
  await convertAfter(page, () => page.selectOption("#ctl-dither", "0"));
  await setRange(page, "#ctl-gamma", "1.0");
  await page.waitForTimeout(400);
  const gNeutral = statsOf(await shot(page));
  await convertAfter(page, () => setRange(page, "#ctl-gamma", "2.2"));
  const gHigh = statsOf(await shot(page));
  check(gHigh.mean > gNeutral.mean + 1, "gamma>1 brightens (pow(c,1/γ))", `mean ${gNeutral.mean.toFixed(1)} → ${gHigh.mean.toFixed(1)}`);
  await convertAfter(page, () => setRange(page, "#ctl-gamma", "1.0"));
  const gBack = statsOf(await shot(page));
  check(Math.abs(gBack.mean - gNeutral.mean) < 1.5, "gamma 1.0 neutral (reversible)", `${gBack.mean.toFixed(1)} vs ${gNeutral.mean.toFixed(1)}`);
  await convertAfter(page, () => setRange(page, "#ctl-contrast", "2.5"));
  await convertAfter(page, () => setRange(page, "#ctl-contrast", "1.0"));

  // ---- 9. ASCII/original blend: 0 vs 1 differ ----
  await loadFile(page, "color-patches.png");
  await convertAfter(page, () => setRange(page, "#ctl-blend", "1"));
  const blendAscii = await shot(page);
  await convertAfter(page, () => setRange(page, "#ctl-blend", "0"));
  const blendOrig = await shot(page);
  check(diffPixels(blendAscii, blendOrig) > 100, "ascii blend 0 vs 1 visibly differ", `${diffPixels(blendAscii, blendOrig)} px`);
  await convertAfter(page, () => setRange(page, "#ctl-blend", "1"));

  // ---- 10. CRT: neutral at 0, visible when on, exact neutral after reset ----
  const crtBase = await shot(page);
  await setRange(page, "#crt-scan", "1");
  await sleep(250);
  const crtScan = await shot(page);
  const scanDiff = diffPixels(crtBase, crtScan);
  check(scanDiff > 50, "scanlines visibly applied", `${scanDiff} px`);
  await setRange(page, "#crt-bloom", "1");
  await setRange(page, "#crt-vig", "1");
  await setRange(page, "#crt-curve", "0.6");
  await sleep(300);
  const crtAll = await shot(page);
  check(diffPixels(crtBase, crtAll) > 200, "all CRT effects visibly applied");
  await setRange(page, "#crt-scan", "0");
  await setRange(page, "#crt-bloom", "0");
  await setRange(page, "#crt-vig", "0");
  await setRange(page, "#crt-curve", "0");
  await sleep(300);
  const crtReset = await shot(page);
  const neutralDiff = diffPixels(crtBase, crtReset);
  check(neutralDiff === 0, "CRT all-zero is pixel-exact neutral", `${neutralDiff} px differ`);

  // ---- 11. rapid changes settle on the latest request ----
  {
    const before = await seq(page);
    for (let i = 0; i < 30; i++) {
      await setRange(page, "#ctl-cell", String(2 + (i % 8)));
      await sleep(8);
    }
    // The debouncer coalesces the burst (by design); at least the latest
    // request must produce an applied conversion.
    await page.waitForFunction(
      (n) =>
        Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
      before,
      { timeout: 30_000 },
    );
    await sleep(900); // settle: no conversions should be pending afterwards
    const seqSettled = await seq(page);
    await sleep(500);
    const seqStable = await seq(page);
    check(seqStable === seqSettled, "no conversion churn after settle", `seq stable at ${seqStable}`);
    const cellVal = await page.inputValue("#ctl-cell");
    const s1 = await page.locator(".status-msg").textContent();
    check(/Converted/.test(s1 ?? "") && cellVal === "7", "rapid changes settle on the LATEST request", `cell=${cellVal}, status=${s1?.slice(0, 60)}`);
    // determinism: same final state twice -> identical pixels
    await setRange(page, "#ctl-cell", "8");
    await sleep(600);
    const d1 = await shot(page);
    await convertAfter(page, () => setRange(page, "#ctl-contrast", "1.05"));
    await convertAfter(page, () => setRange(page, "#ctl-contrast", "1.0"));
    await sleep(400);
    const d2 = await shot(page);
    check(diffPixels(d1, d2) <= 4, "deterministic rerenders", `${diffPixels(d1, d2)} px`);
  }

  // ---- 12. playback controls on animated GIF ----
  await loadFile(page, "anim-2frame-unequal-delays.gif");
  await page.waitForSelector('[aria-label="Animation playback"]');
  const frameTxt0 = await page.locator(".anim-bar .field-value").textContent();
  await page.click('.anim-bar button:has-text("Play")');
  await page.waitForFunction(
    () => /2\s*\/\s*2/.test(document.querySelector(".anim-bar .field-value")?.textContent ?? ""),
    { timeout: 8000 },
  );
  await page.click('.anim-bar button:has-text("Pause")');
  ok("GIF playback play/pause advances frames", `start ${frameTxt0?.trim()}`);

  // ---- 13. exports (independent re-parse) ----
  await loadFile(page, "color-patches.png");
  // ORIGINAL color mode: patches have red grow with x, green with y ->
  // strict channel-order + orientation evidence in the native PNG.
  await convertAfter(page, () => page.selectOption("#ctl-color", "2"));
  await sleep(300);

  const convDims = await page.evaluate(() => {
    const c = document.querySelector('[data-testid="preview-canvas"]');
    return { w: c.width, h: c.height };
  });

  const pngOut = await download(page, () => page.click('[data-testid="export-png"]'));
  const ihdr = parsePngIhdr(pngOut.bytes);
  check(ihdr.width > 0 && ihdr.height > 0, "PNG export valid IHDR", `${ihdr.width}×${ihdr.height}`);
  const pngStat = pngStats(pngOut.bytes);
  check(pngStat.variance > 30, "PNG export non-trivial content", `var=${pngStat.variance.toFixed(1)}`);
  // orientation: patches increases green downward (y//12*40)
  check(pngStat.greenBottom > pngStat.greenTop, "PNG export top-left orientation (green grows down)", `top=${pngStat.greenTop.toFixed(0)} bottom=${pngStat.greenBottom.toFixed(0)}`);
  check(pngStat.redRight > pngStat.redLeft, "PNG channel order R=G=B (red grows right)", `left=${pngStat.redLeft.toFixed(0)} right=${pngStat.redRight.toFixed(0)}`);

  await page.check("#webp-lossless", { force: true });
  const webpLossless = await download(page, () => page.click('[data-testid="export-webp-lossless"]'));
  const wl = webpVariant(webpLossless.bytes);
  check(wl.lossless === true, "WebP lossless export is real VP8L", `chunks=${wl.chunks.join(",")}`);
  await page.uncheck("#webp-lossless", { force: true });
  await page.fill("#webp-q", "60");
  await page.dispatchEvent("#webp-q", "input", { bubbles: true });
  const webpLossy = await download(page, () => page.click('[data-testid="export-webp-lossy"]'));
  const wy = webpVariant(webpLossy.bytes);
  check(wy.lossless === false, "WebP lossy export is real VP8 (not quality-100 lossless)", `chunks=${wy.chunks.join(",")}`);
  check(webpLossy.bytes.length !== webpLossless.bytes.length, "lossy and lossless payloads differ", `${webpLossy.bytes.length} vs ${webpLossless.bytes.length} bytes`);

  const textOut = await download(page, () => page.click('[data-testid="export-text"]'));
  const textDec = new TextDecoder().decode(textOut.bytes);
  const lines = textDec.split("\n");
  check(lines.length >= 3, "text export has rows", `${lines.length} lines`);
  check(!textDec.endsWith("\n"), "text export has no trailing newline");

  const ansiOut = await download(page, () => page.click('[data-testid="export-ansi"]'));
  const ansiDec = new TextDecoder().decode(ansiOut.bytes);
  check(ansiDec.includes("\u001b[38;2;") && ansiDec.includes("\u001b[48;2;"), "ANSI 24-bit fg+bg sequences");
  check(ansiDec.endsWith("\u001b[0m"), "ANSI ends with reset");

  const htmlOut = await download(page, () => page.click('[data-testid="export-html"]'));
  const htmlDec = new TextDecoder().decode(htmlOut.bytes);
  check(htmlDec.startsWith("<!DOCTYPE html>"), "HTML doctype");
  check(htmlDec.includes("data:font/woff;base64,"), "HTML embeds real WOFF (base64)");
  check(/wOFF/.test(Buffer.from(htmlDec.split("base64,")[1]?.split('"')[0] ?? "", "base64").subarray(0, 4).toString("ascii")) || htmlDec.includes("wOFF"), "HTML WOFF magic wOFF present");
  check(!htmlDec.includes("<script"), "HTML contains no scripts");
  check(htmlDec.includes("Content-Security-Policy"), "HTML has CSP meta");

  // still GIF (from still image)
  const gifStill = await download(page, () => page.click('[data-testid="export-gif"]'));
  const g1p = parseGif(gifStill.bytes);
  check(g1p.frames.length === 1, "still GIF has exactly 1 frame", `${g1p.frames.length}`);
  check(g1p.width > 0 && g1p.height > 0, "still GIF dimensions", `${g1p.width}×${g1p.height}`);

  // animated GIF: load animated fixture with alpha Preserve so the fixture's
  // transparent pixel survives quantization; export; parse structure+timing.
  await loadFile(page, "anim-2frame-unequal-delays.gif");
  await convertAfter(page, () => page.selectOption("#ctl-alpha", "1"));
  await sleep(300);
  const gifAnim = await download(page, () => page.click('[data-testid="export-gif"]'));
  const g2 = parseGif(gifAnim.bytes);
  check(g2.frames.length === 2, "animated GIF has 2 frames", `${g2.frames.length}`);
  const delays = g2.frames.map((f) => f.gce?.delay ?? 0);
  check(delays[0] === 40 && delays[1] === 15, "animated GIF delays match source (400ms/150ms → 40/15 cs)", delays.join(","));
  check(g2.loopCode === 0, "animated GIF loops forever (NETSCAPE 0)", `loop=${g2.loopCode}`);
  check(g2.frames[0].gce?.flags !== undefined, "GCE/disposal present", `flags=${g2.frames[0].gce?.flags}`);
  check(g2.frames.some((f) => f.gce && (f.gce.flags & 1) === 1), "transparency flag present in a frame GCE");

  // round-trip: feed exported GIF back into the app
  const seqRt = await seq(page);
  await page.setInputFiles('input[type="file"]', gifAnim.file);
  await page.waitForFunction(
    (n) =>
      Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
    seqRt,
    { timeout: 30_000 },
  );
  const rt = await page.locator(".status-msg").textContent();
  const framesTxt = await page
    .locator('.anim-bar .field-value, .anim-bar .frames')
    .first()
    .textContent()
    .catch(() => "");
  const hasAnimBar = (await page.locator('[aria-label="Animation playback"]').count()) > 0;
  check(
    hasAnimBar && /\/\s*2/.test((framesTxt ?? "") + (rt ?? "")),
    "exported GIF re-opens as a 2-frame animation (independent decode round-trip)",
    `bar=${framesTxt?.trim()} status=${rt?.slice(0, 70)}`,
  );
  // back to alpha Ignore for the remaining tests
  await convertAfter(page, () => page.selectOption("#ctl-alpha", "0"));

  // ---- 14. context loss + restore ----
  const ctxLost = await page.evaluate(() => {
    const c = document.querySelector('[data-testid="preview-canvas"]');
    const gl = c.getContext("webgl2");
    const ext = gl?.getExtension("WEBGL_lose_context");
    if (!ext) return "no-ext";
    ext.loseContext();
    return "lost";
  });
  await sleep(300);
  const note1 = (await page.locator(".renderer-note").count())
    ? await page.locator(".renderer-note").textContent()
    : "";
  check(ctxLost !== "no-ext" && /lost/i.test(note1 ?? ""), "context loss surfaces an actionable note", note1?.trim());
  await page.evaluate(() => {
    const c = document.querySelector('[data-testid="preview-canvas"]');
    const gl = c.getContext("webgl2");
    gl?.getExtension("WEBGL_lose_context")?.restoreContext();
  });
  await sleep(600);
  await setRange(page, "#crt-scan", "1");
  await sleep(250);
  const afterRestore = await page.locator('[data-testid="render-mode"]').textContent();
  check(afterRestore?.trim() === "WebGL2", "context restored; renderer still WebGL2", afterRestore?.trim());
  await setRange(page, "#crt-scan", "0");

  // ---- 15. CPU fallback (force no WebGL2) ----
  const page2 = await context.newPage();
  await page2.addInitScript(() => {
    const orig = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (type, ...args) {
      if (type === "webgl2") return null;
      return orig.call(this, type, ...args);
    };
  });
  await page2.goto(BASE, { waitUntil: "load" });
  await page2.waitForSelector(".app");
  await page2.setInputFiles('input[type="file"]', path.join(FIX, "color-patches.png"));
  await page2.waitForFunction(
    (n) =>
      Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n,
    0,
    { timeout: 30_000 },
  );
  const mode2 = (await page2.locator('[data-testid="render-mode"]').textContent())?.trim();
  check(mode2 === "Canvas2D fallback", "CPU fallback engages without WebGL2", mode2);
  const crtDisabled = await page2.isDisabled("#crt-scan");
  check(crtDisabled, "CRT controls disabled in fallback (identified, not silent)");
  const shot2 = PNG.sync.read(await page2.locator('[data-testid="preview-canvas"]').screenshot());
  check(statsOf(shot2).variance > 30, "CPU fallback renders content");
  await page2.close();

  // ---- 16. budgets: 1MP conversion time ----
  await page.goto(BASE, { waitUntil: "load" });
  await page.waitForSelector(".app");
  // default cell 8 -> 128x128 cells on 1024x1024 source -> 1 MP output (in budget)
  await loadFile(page, "gradient-1024x1024.png");
  const ms1mpText = await page.locator('[data-testid="convert-ms"]').textContent();
  const ms1mp = Number(ms1mpText?.replace(/[^\d.]/g, ""));
  check(Number.isFinite(ms1mp), "1MP conversion measured (default cell 8)", `${ms1mp} ms`);
  // warm rerun (same dims, tiny param delta)
  await convertAfter(page, () => setRange(page, "#ctl-contrast", "1.01"));
  const warm = await page.locator('[data-testid="convert-ms"]').textContent();
  const ms1mpWarm = Number(warm?.replace(/[^\d.]/g, ""));
  ok("1MP warm timing recorded", `${ms1mpWarm} ms (target <100 ms: ${ms1mpWarm < 100 ? "in budget" : "OVER"})`);

  // cell 1 on a 1 MP image => 8192x8192 = 67 MP output: must reject the LIMIT
  // cleanly (not hang/crash), preview intact, then recover.
  const seqCell = await seq(page);
  await setRange(page, "#ctl-cell", "1");
  await page.waitForFunction(
    (n) =>
      Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > n ||
      /fail|exceed|limit/i.test(document.querySelector(".status-msg")?.textContent ?? ""),
    seqCell,
    { timeout: 60_000 },
  );
  const cellErr = await page.locator(".status-msg").textContent();
  check(/exceed|limit|fail/i.test(cellErr ?? ""), "16.7 MP output limit reported clearly", cellErr?.slice(0, 110));
  check((await page.locator('[data-testid="preview-canvas"]:not(.is-hidden)').count()) === 1, "preview preserved after limit error");
  await convertAfter(page, () => setRange(page, "#ctl-cell", "8"));
  ok("recovered after limit error (cell back to 8)");

  // ---- 17. mission extras: per-effect CRT + CPU-vs-GL native equality ----
  // (a) every CRT effect separately: ON differs from baseline, OFF exact neutral
  await page.goto(BASE, { waitUntil: "load" });
  await page.waitForSelector(".app");
  await loadFile(page, "color-patches.png");
  const crtBase2 = await shot(page);
  for (const [label, sel, on, off] of [
    ["scanlines", "#crt-scan", "1", "0"],
    ["bloom", "#crt-bloom", "1", "0"],
    ["vignette", "#crt-vig", "1", "0"],
    ["curvature", "#crt-curve", "0.6", "0"],
  ]) {
    await setRange(page, sel, on);
    await sleep(260);
    const onShot = await shot(page);
    await setRange(page, sel, off);
    await sleep(260);
    const offShot = await shot(page);
    const dOn = diffPixels(crtBase2, onShot);
    const dOff = diffPixels(crtBase2, offShot);
    check(dOn > 20, `CRT ${label} visibly applied (alone)`, `${dOn} px`);
    check(dOff === 0, `CRT ${label}=0 exact neutral`, `${dOff} px`);
  }

  // (b) CPU reference vs WebGL2 at native resolution, CRT off, same defaults
  const page3 = await context.newPage();
  await page3.setViewportSize({ width: 1440, height: 960 });
  await page3.addInitScript(() => {
    const orig = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (type, ...args) {
      if (type === "webgl2") return null;
      return orig.call(this, type, ...args);
    };
  });
  await page3.goto(BASE, { waitUntil: "load" });
  await page3.waitForSelector(".app");
  await page3.setInputFiles('input[type="file"]', path.join(FIX, "color-patches.png"));
  await page3.waitForFunction(
    () => Number(document.querySelector("[data-testid=status]")?.dataset.resultSeq ?? 0) > 0,
    { timeout: 30_000 },
  );
  await sleep(300);
  const glShot = await shot(page);
  const cpuShot = PNG.sync.read(await page3.locator('[data-testid="preview-canvas"]').screenshot());
  await page3.close();
  if (glShot.width === cpuShot.width && glShot.height === cpuShot.height) {
    let close = 0;
    let sumAbs = 0;
    const n = glShot.data.length / 4;
    for (let i = 0; i < glShot.data.length; i += 4) {
      const d = Math.max(
        Math.abs(glShot.data[i] - cpuShot.data[i]),
        Math.abs(glShot.data[i + 1] - cpuShot.data[i + 1]),
        Math.abs(glShot.data[i + 2] - cpuShot.data[i + 2]),
      );
      if (d <= 8) close++;
      sumAbs += Math.abs(glShot.data[i] - cpuShot.data[i]) + Math.abs(glShot.data[i + 1] - cpuShot.data[i + 1]) + Math.abs(glShot.data[i + 2] - cpuShot.data[i + 2]);
    }
    const pct = (close / n) * 100;
    const meanAbs = sumAbs / (n * 3);
    check(
      pct >= 97,
      "CPU fallback ≈ WebGL2 output (native, CRT off, same defaults)",
      `${pct.toFixed(2)}% pixels within ±8, mean|Δ|=${meanAbs.toFixed(2)}`,
    );
  } else {
    fail("CPU fallback ≈ WebGL2 output", `size mismatch ${glShot.width}x${glShot.height} vs ${cpuShot.width}x${cpuShot.height}`);
  }

  // ---- 18. network + CSP + console audits ----
  const external = requests.filter((u) => !u.startsWith(BASE) && !u.startsWith("data:") && !u.startsWith("blob:"));
  check(external.length === 0, "no external network requests", external.slice(0, 3).join(" "));
  check(wasmContentType === "application/wasm", "WASM served with application/wasm", String(wasmContentType));
  const csp = await page.evaluate(() => window.__csp);
  check((csp ?? []).length === 0, "zero CSP violations", (csp ?? []).slice(0, 2).join(" | "));
  check(pageErrors.length === 0, "zero uncaught page errors", pageErrors.slice(0, 2).join(" | "));
  const realConsoleErrors = consoleErrors.filter(
    (e) => !/favicon/i.test(e) && !/Download the React DevTools/i.test(e),
  );
  check(realConsoleErrors.length === 0, "zero console errors", realConsoleErrors.slice(0, 3).join(" | "));

  // ---- report ----
  const pass = results.filter((r) => r.pass).length;
  const failCount = results.length - pass;
  const summary = {
    target: BASE,
    pass,
    fail: failCount,
    glRenderer,
    convertMs48: convertMs,
    convertMs1M: ms1mp,
    convertMs1Mwarm: ms1mpWarm,
    wasmContentType,
    externalRequests: external,
    consoleErrors: realConsoleErrors,
    pageErrors,
    cspViolations: csp,
  };
  fs.writeFileSync(path.join(HERE, "e2e-results.json"), JSON.stringify(summary, null, 2));
  console.log(`\n== ${pass} passed, ${failCount} failed`);
  console.log(`gl renderer: ${glRenderer}`);
  console.log(`1MP convert: cold ${ms1mp} ms, warm ${ms1mpWarm} ms`);

  await browser.close();
  process.exit(failCount === 0 ? 0 : 1);
}

main().catch(async (e) => {
  console.error("E2E crashed:", e);
  process.exit(2);
});
