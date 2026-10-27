import { createSignal, onMount, createEffect } from 'solid-js';
import { useUnscii } from './hooks/useUnscii';
import { useImage } from './hooks/useImage';
import { useWebGL } from './hooks/useWebGL';
import { useExport } from './hooks/useExport';
import { params, setParams } from './stores/params';
import { ConvertParams, AsciiGrid, ExportParams } from 'unscii-converter-core';
import { FontVariant } from 'unscii-converter-core';
import Canvas from './components/Canvas';
import ControlPanel from './components/ControlPanel';
import FileDrop from './components/FileDrop';
import ExportModal from './components/ExportModal';
import StatusBar from './components/StatusBar';

export default function App() {
  const [showExport, setShowExport] = createSignal(false);
  const [statusMessage, setStatusMessage] = createSignal('Drop an image or click to load');
  
  const { converter, ready, init, setFontVariant, currentVariant, getFontAtlas, getFontAtlasSize, getFontVariant } = useUnscii();
  const { loadImage, clearImage, imageData, imageName } = useImage();
  const { 
    canvasRef, 
    renderFrame, 
    resizeCanvas,
    gl,
    error: glError
  } = useWebGL();
  // Store last converted grid for export
  const [lastGrid, setLastGrid] = createSignal<AsciiGrid | null>(null);
  // Pass converter and grid signals to useExport
  const { exportAll, downloading } = useExport(converter, lastGrid);

  const handleFileLoad = async (file: File) => {
    console.log('handleFileLoad called with:', file);
    const conv = converter();
    console.log('converter():', conv, 'ready():', ready());
    if (!conv || !ready()) {
      setStatusMessage('WASM not ready yet...');
      return;
    }
    
    setStatusMessage(`Loading ${file.name}...`);
    try {
      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      // Load and decode image once in WASM
      conv.load_image(bytes);
      // Also store locally for UI state
      await loadImage(bytes, file.name);
      setStatusMessage(`Loaded ${file.name} - ready to convert`);
    } catch (e) {
      setStatusMessage(`Error: ${e instanceof Error ? e.message : 'Failed to load'}`);
    }
  };

  // Debug: log when signals change
  createEffect(() => {
    console.log('converter changed:', converter());
  });
  createEffect(() => {
    console.log('ready changed:', ready());
  });
  createEffect(() => {
    console.log('imageData changed:', imageData());
  });

  // Auto-convert when params change (debounced)
  const doConvert = async () => {
    const conv = converter();
    if (!conv || !ready() || !imageData()) return;
    
    setStatusMessage('Converting...');
    try {
      resizeCanvas();
      const p = params;
      const wasmParams = new ConvertParams();
      wasmParams.dither_algorithm = p.ditherAlgorithm;
      wasmParams.dither_strength = p.ditherStrength;
      wasmParams.threshold = p.threshold;
      wasmParams.font_variant = p.fontVariant;
      wasmParams.cell_size = p.cellSize;
      wasmParams.color_mode = p.colorMode;
      wasmParams.palette = new Uint32Array(p.palette);
      wasmParams.palette_size = p.paletteSize;
      wasmParams.alpha_mode = p.alphaMode;
      wasmParams.contrast = p.contrast;
      wasmParams.gamma = p.gamma;
      wasmParams.target_width = p.targetWidth;
      wasmParams.crt_scanlines = p.crtScanlines;
      wasmParams.crt_curvature = p.crtCurvature;
      wasmParams.crt_bloom = p.crtBloom;
      wasmParams.crt_vignette = p.crtVignette;
      // Use convert_loaded which skips decoding (image already loaded in WASM)
      const result = conv.convert_loaded(wasmParams);
      const grid = result instanceof Promise ? await result : result;
      // Get glyph dimensions from converter's font atlas
      const atlasSize = getFontAtlasSize ? getFontAtlasSize() : [128, 8];
      const glyphWidth = atlasSize[0] / 16; // 16 glyphs horizontally
      const glyphHeight = atlasSize[1];
      console.log('Glyph dimensions:', glyphWidth, 'x', glyphHeight);
      renderFrame(grid, glyphWidth, glyphHeight);
      setLastGrid(grid); // Store for export
      setStatusMessage(`Converted: ${grid.cols()}×${grid.rows()} cells`);
    } catch (e) {
      setStatusMessage(`Error: ${e instanceof Error ? e.message : 'Conversion failed'}`);
    }
  };

  // Track params for auto-convert (access all reactive values)
  createEffect(() => {
    const p = params;
    // Access all params to track them
    p.ditherAlgorithm;
    p.ditherStrength;
    p.threshold;
    p.fontVariant;
    p.cellSize;
    p.colorMode;
    p.palette;
    p.paletteSize;
    p.alphaMode;
    p.contrast;
    p.gamma;
    p.targetWidth;
    p.crtScanlines;
    p.crtCurvature;
    p.crtBloom;
    p.crtVignette;
    
    console.log('params effect triggered:', p.ditherStrength, p.threshold);
    if (converter() && ready() && imageData()) {
      console.log('Calling doConvert from effect');
      doConvert();
    }
  });

  const handleConvert = async () => {
    await doConvert();
  };

  // Track font variant changes and trigger full refresh
  createEffect(() => {
    const variant = params.fontVariant;
    console.log('fontVariant changed:', variant);
    if (converter() && ready() && currentVariant() !== variant) {
      // Use async to let the variant change complete before converting
      setFontVariant(variant).then(() => {
        // After font variant is set and atlas uploaded, re-convert
        if (imageData()) {
          doConvert();
        }
      });
    }
  });

  const handleExport = async () => {
    const conv = converter();
    if (!conv || !ready()) return;
    try {
      setShowExport(true);
    } catch (e) {
      console.error(e);
    }
  };

  const handleExportConfirm = async (exportParams: any) => {
    const conv = converter();
    if (!conv || !ready()) return;
    try {
      setStatusMessage('Exporting...');
      // Create proper ExportParams from UI values
      const wasmExportParams = new ExportParams();
      wasmExportParams.webp_quality = exportParams.webpQuality ?? 85;
      wasmExportParams.webp_lossless = exportParams.webpLossless ?? false;
      wasmExportParams.html_fg = exportParams.htmlFg ?? '#ffffff';
      wasmExportParams.html_bg = exportParams.htmlBg ?? '#000000';
      wasmExportParams.html_font = exportParams.htmlFont ?? "'Unscii', monospace";
      wasmExportParams.html_line_height = exportParams.htmlLineHeight ?? 1.1;
      wasmExportParams.html_font_size = exportParams.htmlFontSize ?? 12;
      // Font data for embedded HTML export (base64 WOFF)
      wasmExportParams.html_font_base64 = ""; // TODO: get from converter
      wasmExportParams.html_font_format = "woff";
      // Get current font variant
      const variantNames = ["Unscii8", "Unscii8Alt", "Unscii8Fantasy", "Unscii8Mcr", "Unscii8Tall", "Unscii8Thin", "Unscii16"];
      wasmExportParams.html_font_variant = variantNames[params.fontVariant] || "Unscii8";
      wasmExportParams.ansi_fg = exportParams.ansiFg ?? 0xFFFFFFFF;
      wasmExportParams.ansi_bg = exportParams.ansiBg ?? 0x000000FF;
      await exportAll(wasmExportParams);
      setShowExport(false);
      setStatusMessage('Export complete!');
    } catch (e) {
      setStatusMessage(`Export error: ${e instanceof Error ? e.message : 'Failed'}`);
    }
  };

  onMount(async () => {
    console.log('onMount called');
    await init();
    console.log('onMount done, ready():', ready());
  });

  return (
    <div class="app">
      <header class="header">
        <h1>Unscii Image Converter</h1>
        <p class="subtitle">Interactive ASCII art with unscii font</p>
      </header>
      
      <main class="main">
        <div class="workspace">
          <Canvas ref={canvasRef} onResize={() => doConvert()} />
          <ControlPanel 
            params={params} 
            setParams={setParams}
            onConvert={handleConvert}
            onExport={handleExport}
            disabled={!ready() || !imageData()}
            converting={false}
          />
        </div>
        
        <FileDrop 
          onLoad={handleFileLoad}
          disabled={!ready()}
          imageName={imageName()}
          onClear={clearImage}
        />
        
        <StatusBar message={statusMessage} glError={glError} />
      </main>
      
      <ExportModal 
        show={showExport} 
        onClose={() => setShowExport(false)}
        onConfirm={handleExportConfirm}
        disabled={downloading()}
      />
    </div>
  );
}