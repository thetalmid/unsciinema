# Unsciinema

Turn images and animations into authentic **Unscii ASCII/ANSI art** — entirely
in the browser. A Rust → WebAssembly conversion core runs inside a Web Worker;
a WebGL2 renderer previews the result live with optional CRT effects; every
advertised export (PNG, text, ANSI, self-contained HTML, GIF, WebP) is produced
locally. No uploads, no analytics, no third-party runtime calls.

## Repository layout

| Path | Contents |
|---|---|
| `core/` | Rust → WASM conversion pipeline (dithering, palettes, fonts, raster, exports). 48 unit tests. |
| `frontend/` | Solid 2 + Vite app: worker protocol, WebGL2 renderer with CPU/Canvas2D fallback, controls, exports. |
| `public/fonts/` | Seven unscii variants: `.hex` bitmaps (embedded in the WASM core) + upstream vectorized `.woff`. |
| `scripts/build-wasm.sh` | `wasm-pack build --target web --release` into `frontend/src/generated/wasm/`. |
| `test-fixtures/` | 14 purpose-built inputs (PNG/JPEG/GIF/WebP, odd sizes, alpha, malformed, oversized, animated with unequal delays + transparency + disposal change). |
| `verify/` | Playwright real-browser E2E (`e2e.mjs`), fixture generator (`make-fixtures.mjs`). |

## Requirements

* Rust (stable) with `wasm32-unknown-unknown` target and `wasm-pack`
* Node.js ≥ 20 (developed on 22)
* A Chromium browser for the E2E suite (`playwright` downloads its own)

## Build and run (documented commands)

```sh
# 1. WASM core tests
cd core && cargo test          # 48 tests, 0 warnings

# 2. Generate the WASM package (from anywhere)
./scripts/build-wasm.sh        # = (cd core && wasm-pack build --target web \
                               #    --out-dir ../frontend/src/generated/wasm --release)

# 3. Frontend: install, type-check, production build
cd frontend
npm install
npm run build                  # tsc --noEmit && vite build

# 4. Static preview (serves the production build)
npm run preview                # http://localhost:4173/
```

### Nested base path

The build is deployable at `/` and at a nested base path; the config reads
`VITE_BASE`:

```sh
cd frontend
VITE_BASE=/nested/path/ npm run build -- --outDir dist-nested
VITE_BASE=/nested/path/ npm run preview -- --outDir dist-nested --port 4174
# → http://localhost:4174/nested/path/
```

Asset, worker, codec, and font URLs are same-origin and base-prefixed; the
WASM responses are served with `Content-Type: application/wasm`.

## Verification (real browser)

```sh
cd verify
npm install
npx playwright install chromium   # once (plus `npx playwright install-deps chromium` on bare Linux)
node e2e.mjs                          # against http://127.0.0.1:4173/
node e2e.mjs http://127.0.0.1:4174/nested/path/   # nested-base deployment
```

`e2e.mjs` exercises every fixture, font, dither algorithm, color/palette/alpha
mode, ASCII-blend and CRT behavior, playback, context loss/restore, the CPU
fallback, determinism, limits/errors, and downloads **every export**, decoding
each result independently (PNG IHDR, GIF frames/delays/loop/disposal/
transparency, WebP `VP8 ` vs `VP8L`, HTML CSP + embedded WOFF, ANSI reset,
text grid). It also audits the network log (no external/`localhost` requests),
CSP violations, console/page errors, and records conversion timings.

Results are written to `verify/e2e-results.json` (and
`verify/e2e-results-nested.json` for the nested run).

## License

Dual-licensed **MIT OR Apache-2.0**: see `LICENSE-MIT` and `LICENSE-APACHE`.
Bundled third-party crates, npm packages, and font provenance (including the
deliberately excluded AGPL packages) are documented in
`THIRD-PARTY-NOTICES.md`.
