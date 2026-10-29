# Third-party notices and provenance

This project (`unsciinema-core`, `unsciinema-frontend`) is dual-licensed
**MIT OR Apache-2.0** (see `LICENSE-MIT` and `LICENSE-APACHE`), copyright
2026 Marcelo Sampaio. This file documents the licenses and provenance of
direct dependencies and bundled assets. License texts for
permissive crates/npm packages are preserved by their own toolchains
(`cargo`, `npm`); the summary table below records what was resolved and why.

## Rust crates (direct dependencies of `core/`)

| Crate | Version | License | Role |
|---|---|---|---|
| `wasm-bindgen` | 0.2.129 | MIT OR Apache-2.0 | WASM bindings |
| `image` | 0.25.10 | MIT OR Apache-2.0 | content-sniffed PNG/JPEG/GIF/WebP decode (`png`, `jpeg`, `webp`, `gif` features) |
| `png` | 0.18.1 | MIT OR Apache-2.0 | PNG encoding for the PNG export |
| `gif` | 0.14.2 | MIT OR Apache-2.0 | GIF encode |
| `gif-dispose` | 6.0.0 | MIT OR Apache-2.0 | GIF frame compositing (disposal/transparency) on decode |
| `color_quant` | 2.0.0 | MIT | NeuQuant quantizer used by the GIF export |
| `kamadak-exif` | 0.6.1 | BSD-2-Clause | EXIF orientation |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | `.hex` font parsing |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | error derive (`core/src/error.rs`) |
| `console_error_panic_hook` | 0.1 | MIT OR Apache-2.0 | readable panic messages in the browser console |

### Deliberately excluded (license-incompatible)

| Package | Why excluded |
|---|---|
| `gifski-wasm` | AGPL-3.0-or-later (project declares MIT OR Apache-2.0) |
| `zengif` (default features → `zenquant`) | AGPL |
| `zenwebp` | AGPL/commercial |
| `quantizr` | AGPL |

## JavaScript (direct dependencies of `frontend/`)

| Package | Version | License | Role |
|---|---|---|---|
| `solid-js` | 2.0.0-rc.19 | MIT | UI runtime |
| `@solidjs/web` | 2.0.0-rc.19 | MIT | DOM renderer |
| `@solidjs/vite-plugin` | 3.0.0-next.52 | MIT | build plugin |
| `vite` | 8.3.5 | MIT | bundler/dev/preview |
| `typescript` | 5.9.3 | Apache-2.0 | type checking |
| `@jsquash/webp` | 1.5.0 | Apache-2.0 | real WebP lossy (VP8) + lossless (VP8L) encode via libwebp WASM, lazy-loaded only on WebP export |

Dev/verification only (`verify/`): `playwright` 1.64.1 (Apache-2.0),
`pngjs` 7.0.0 (MIT). These are not part of the shipped static build.

All JS dependencies are bundled from `node_modules` at build time; the
shipped site loads no third-party scripts, fonts, or codecs from the network.

## Fonts (bundled assets)

* Source project: **unscii** by viznut (https://viznut.fi/unscii/).
* Seven variants shipped: `unscii-8`, `unscii-8-alt`, `unscii-8-fantasy`,
  `unscii-8-mcr`, `unscii-8-tall`, `unscii-8-thin`, `unscii-16`.
  The unscii project places these variants in the **public domain**.
  The differently-licensed `unscii-16-full` (contains GPL Unifont pieces) is
  **not shipped**.
* Two forms are bundled:
  * `public/fonts/*.hex` — original bitmap records, embedded into the WASM
    core via `include_str!` (conversion pipeline input).
  * `public/fonts/*.woff` — the upstream-published **vectorized** WOFFs of the
    same seven variants (real WOFF magic, used by the HTML export and UI font
    loading). These are the upstream files, downloaded from viznut.fi; they
    are vectorizations of the public-domain bitmaps.
* No remote font fetches occur at runtime; the HTML export embeds the WOFF as
  a `data:font/woff;base64,` URI.

## Quantization / partial-alpha policy (GIF export)

Documented in `core/src/export/gif.rs`: pixels with alpha < 128 map to a
reserved transparent index; opaque pixels are quantized with NeuQuant
(`color_quant`, MIT) trained only on opaque pixels when transparency exists
(255-color palette + transparent slot); RGB is taken straight (no
premultiplication); partial alpha is binary in GIF (opaque above the cut);
disposal is `Background` on written frames so transparency never leaks
between frames. GIF cannot carry full RGBA alpha and none is promised.
