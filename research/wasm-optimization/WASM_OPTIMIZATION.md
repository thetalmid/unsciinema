# WASM Bundle Size Optimization

## Target
- **< 500 KB gzipped** (ideal: 200-300 KB)
- **< 2 MB raw** (before gzip)

## Rust Compilation Flags

### Cargo.toml Release Profile
```toml
[profile.release]
opt-level = "z"           # Optimize for size (not speed)
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit
panic = "abort"           # Smaller than unwind
strip = true              # Strip symbols
debug = false             # No debug info
overflow-checks = false   # Disable overflow checks (slight risk)
```

### .cargo/config.toml
```toml
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "link-arg=--strip-all",           # Strip all symbols
    "-C", "link-arg=--no-entry",            # No entry point (wasm-bindgen provides)
    "-C", "link-arg=--export-table",        # Export table for wasm-bindgen
    "-C", "link-arg=--allow-undefined",     # Allow undefined imports
    "-C", "target-feature=+simd128",        # Enable SIMD128 for zenwebp
    "-C", "target-feature=+bulk-memory",    # Bulk memory operations
    "-C", "target-feature=+mutable-globals", # Mutable globals
]

[profile.release]
# Override for wasm-pack
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## wasm-pack Configuration
```toml
# Cargo.toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = true           # Run wasm-opt automatically (wasm-pack 0.12+)
wasm-opt-args = ["-Oz", "--enable-bulk-memory", "--enable-simd"]
```

## wasm-opt (Binaryen) — Post-Build Optimization
```bash
# Install
npm install -g wasm-opt
# or
cargo install wasm-opt  # Rust wrapper

# Run manually
wasm-opt -Oz \
  --enable-bulk-memory \
  --enable-simd \
  --strip-all \
  -o pkg/unscii_converter_core_bg_opt.wasm \
  pkg/unscii_converter_core_bg.wasm
```

### wasm-opt Passes (for size)
| Pass | Flag | Effect |
|------|------|--------|
| Optimize | `-O` / `-Oz` | General optimization (size-focused) |
| Bulk memory | `--enable-bulk-memory` | memory.copy/fill/init |
| SIMD | `--enable-simd` | SIMD128 instructions |
| Strip | `--strip-all` | Remove names, debug info |
| Converge | `--converge` | Run until fixed point |

## Code-Level Optimizations

### 1. Minimize Dependencies
```toml
# Use minimal feature sets
image = { version = "0.25", features = ["png", "jpeg", "gif", "bmp"], default-features = false }
# webp via zenwebp (pure Rust), not image-webp (may pull libwebp-sys)
```

### 2. Avoid Large Std Components
```rust
// ❌ Pulls in formatting machinery
format!("{}", value)

// ✅ Use core::fmt::Write or write! macro sparingly
// ✅ For logging: use `console_error_panic_hook` only in debug
```

### 3. Use `wee_alloc` (Optional, Small Allocator)
```toml
[dependencies]
wee_alloc = { version = "0.4", optional = true }

[features]
default = ["wee_alloc"]
```
```rust
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```
**Note:** Test if it actually reduces size; modern `dlmalloc` in std is competitive.

### 4. Avoid `serde` for WASM Exports
```rust
// ❌ serde pulls in ~100KB
#[wasm_bindgen]
pub fn process(json_config: &str) -> String { ... }

// ✅ Use wasm-bindgen native types
#[wasm_bindgen]
pub fn process(config: &ConvertParams) -> Result<Vec<u8>, JsValue> { ... }
```

### 5. Inline Small Functions
```rust
#[inline(always)]
fn luminance(r: u8, g: u8, b: u8) -> u8 { ... }
```

### 6. Use `Vec::with_capacity` to Avoid Reallocation Overhead
```rust
let mut out = Vec::with_capacity(width * height * 4);
```

### 7. Feature-Gate Optional Code
```toml
[features]
default = ["png", "jpeg"]
gif = ["image/gif"]
webp = ["zenwebp"]
export = ["png", "webp", "gif"]
```

## Dependency Analysis
```bash
# Check crate sizes
cargo bloat --release -t wasm32-unknown-unknown

# Or
cargo install cargo-bloat
cargo bloat --release --crates
```

### Typical Size Contributors
| Crate | Est. Size (raw) | Notes |
|-------|-----------------|-------|
| `wasm-bindgen` | ~50 KB | Essential |
| `web-sys` | ~30 KB | Only enable needed features |
| `image` (png/jpeg) | ~200 KB | Core |
| `zenwebp` | ~150 KB | Pure Rust WebP |
| `gifski-lite` | ~300 KB | Consider JS-side instead |
| `png` encoder | ~50 KB | |
| `std` (panic=abort) | ~100 KB | Minimal |

## Bundle Analysis Workflow
```bash
# 1. Build with wasm-pack
wasm-pack build --release --target web --out-dir pkg

# 2. Check raw size
ls -lh pkg/*.wasm

# 3. Run wasm-opt manually
wasm-opt -Oz --enable-bulk-memory --enable-simd --strip-all \
  -o pkg/unscii_converter_core_bg_opt.wasm pkg/unscii_converter_core_bg.wasm

# 4. Check optimized size
ls -lh pkg/*_opt.wasm

# 5. Gzip test
gzip -c pkg/unscii_converter_core_bg_opt.wasm | wc -c

# 6. Analyze with twiggy (if needed)
cargo install twiggy
twiggy top pkg/unscii_converter_core_bg_opt.wasm
twiggy dominators pkg/unscii_converter_core_bg_opt.wasm
```

## Vite Build Optimization
```typescript
// vite.config.ts
export default defineConfig({
  build: {
    target: 'esnext',
    minify: 'esbuild',  // Faster than terser
    cssCodeSplit: false,
    rollupOptions: {
      output: {
        manualChunks: undefined,  // Single bundle for WASM app
      },
    },
  },
  // ESBuild handles WASM via vite-plugin-wasm
});
```

## Font Files (unscii)
- **Don't bundle in WASM** — load at runtime via `fetch()`
- **Cache in IndexedDB** for offline
- **Size:** unscii-8.hex ≈ 73 KB raw → ~15 KB gzipped
- **WOFF font:** ~15 KB gzipped (for HTML export)

## Runtime Memory Management
```rust
// Reuse buffers instead of allocating
thread_local! {
    static SCRATCH_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

fn process_image(input: &[u8]) -> Vec<u8> {
    SCRATCH_BUFFER.with(|buf| {
        let mut scratch = buf.borrow_mut();
        scratch.clear();
        scratch.extend_from_slice(input);
        // ... process in place
        scratch.clone()
    })
}
```

## Lazy Loading (Code Splitting)
```typescript
// Load WASM only when needed
const loadConverter = async () => {
  const { default: init, UnsciiConverter } = await import('./pkg/unscii_converter_core.js');
  await init();
  return UnsciiConverter;
};

// In component
onMount(async () => {
  const Converter = await loadConverter();
  // ...
});
```

## Monitoring Size in CI
```yaml
# .github/workflows/size.yml
- name: Build WASM
  run: wasm-pack build --release

- name: Check size
  run: |
    SIZE=$(stat -c%s pkg/unscii_converter_core_bg.wasm)
    GZIP_SIZE=$(gzip -c pkg/unscii_converter_core_bg.wasm | wc -c)
    echo "Raw: $SIZE bytes"
    echo "Gzipped: $GZIP_SIZE bytes"
    if [ $GZIP_SIZE -gt 500000 ]; then
      echo "❌ Gzipped size exceeds 500KB!"
      exit 1
    fi
```

## References
- **Rust WASM Book - Code Size**: https://rustwasm.github.io/docs/book/reference/code-size.html
- **wasm-bindgen Guide - Optimizing for Size**: https://wasm-bindgen.github.io/wasm-bindgen/reference/optimize-size.html
- **wasm-pack Cargo.toml Config**: https://wasm-bindgen.github.io/wasm-pack/book/cargo-toml-configuration.html
- **Binaryen wasm-opt**: https://github.com/WebAssembly/binaryen
- **Reducing WASM Bundle Size with wasm-opt**: https://www.webassembly-wasm.com/compilation-pipelines-toolchain-setup/wasm-optimization-flags-size-reduction/reducing-wasm-bundle-size-with-wasm-opt/
- **wasm-pack auto wasm-opt**: https://github.com/rustwasm/wasm-pack/pull/625