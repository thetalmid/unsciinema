import { createStore, produce } from 'solid-js/store';
import { createMemo } from 'solid-js';

export interface ConvertParams {
  // Dithering
  ditherAlgorithm: number;
  ditherStrength: number;
  threshold: number;
  
  // Font
  fontVariant: number;
  
  // Rendering
  cellSize: number;
  colorMode: number;
  palette: number[];
  paletteSize: number;
  
  // Alpha
  alphaMode: number;
  
  // Pre-processing
  contrast: number;
  gamma: number;
  targetWidth: number;
  
  // CRT effects
  crtScanlines: boolean;
  crtCurvature: number;
  crtBloom: number;
  crtVignette: number;
}

export const [params, setParams] = createStore<ConvertParams>({
  ditherAlgorithm: 0,
  ditherStrength: 1.0,
  threshold: 0.5,
  fontVariant: 0,
  cellSize: 8.0,
  colorMode: 0,
  // 8-digit hex with alpha (RRGGBBAA)
  palette: [0x000000FF, 0xFFFFFFFF],
  paletteSize: 2,
  alphaMode: 0,
  contrast: 1.0,
  gamma: 1.0,
  targetWidth: 0,
  crtScanlines: false,
  crtCurvature: 0.0,
  crtBloom: 0.0,
  crtVignette: 0.0,
});

export const DITHER_ALGORITHMS = [
  { value: 0, label: 'None' },
  { value: 1, label: 'Floyd-Steinberg' },
  { value: 2, label: 'Atkinson' },
  { value: 3, label: 'Bayer 4×4' },
  { value: 4, label: 'Bayer 8×8' },
  { value: 5, label: 'Blue Noise' },
  { value: 6, label: 'Sierra' },
  { value: 7, label: 'Jarvis-Judice-Ninke' },
  { value: 8, label: 'Stucki' },
  { value: 9, label: 'Burkes' },
];

export const FONT_VARIANTS = [
  { value: 0, label: 'unscii-8' },
  { value: 1, label: 'unscii-8-alt' },
  { value: 2, label: 'unscii-8-fantasy' },
  { value: 3, label: 'unscii-8-mcr' },
  { value: 4, label: 'unscii-8-tall' },
  { value: 5, label: 'unscii-8-thin' },
  { value: 6, label: 'unscii-16' },
];

export const COLOR_MODES = [
  { value: 0, label: 'Monochrome' },
  { value: 1, label: 'Color' },
  { value: 2, label: 'Palette' },
];

export const ALPHA_MODES = [
  { value: 0, label: 'Ignore' },
  { value: 1, label: 'Transparent' },
  { value: 2, label: 'Blend' },
];

// All 31 palettes synced from Rust core/src/dither/palette.rs
// Format: 8-digit hex RRGGBBAA
export const PALETTE_PRESETS = [
  // Monochrome
  { name: 'monochrome', colors: [0x000000FF, 0xFFFFFFFF] },
  { name: 'terminal', colors: [0x000000FF, 0x333333FF, 0x666666FF, 0x999999FF, 0xCCCCCCFF, 0xFFFFFFFF] },
  { name: 'gameboy', colors: [0x0F380FFF, 0x306230FF, 0x8BAC0FFF, 0x9BBC0FFF] },
  
  // Warm
  { name: 'sepia', colors: [0x1A1A1AFF, 0x4A3A2AFF, 0x8A6A4AFF, 0xD4B896FF, 0xF5E6C8FF] },
  { name: 'campfire', colors: [0x0D0D0DFF, 0x3D1A0DFF, 0x8B3A1AFF, 0xD47A3AFF, 0xF5C896FF] },
  { name: 'gold', colors: [0x0D0D00FF, 0x3D3D00FF, 0xB8B800FF, 0xFFF000FF, 0xFFFF80FF] },
  { name: 'autumn', colors: [0x1A0D00FF, 0x4A1A00FF, 0x8B3A00FF, 0xD46A00FF, 0xF5A600FF] },
  
  // Cool
  { name: 'ocean', colors: [0x000D1AFF, 0x003D4DFF, 0x007A8CFF, 0x4DB8D4FF, 0x96E6F5FF] },
  { name: 'ice', colors: [0x00001AFF, 0x002A6DFF, 0x006AB8FF, 0x4DA6E6FF, 0x96D4F5FF] },
  { name: 'midnight', colors: [0x00001AFF, 0x001A4DFF, 0x003A8CFF, 0x4A6AD4FF, 0x9696F5FF] },
  
  // Neon / Synthwave
  { name: 'synthwave', colors: [0x1A0033FF, 0x4D0066FF, 0xFF00FFFF, 0xFF66FFFF, 0xFFCCFFFF] },
  { name: 'cyberpunk', colors: [0x0D001AFF, 0x3D0066FF, 0xB800FFFF, 0xFF00FFFF, 0xFF80FFFF] },
  { name: 'neon', colors: [0x00001AFF, 0x004D00FF, 0x00FF00FF, 0x00FF80FF, 0x80FFFF80] },
  { name: 'matrix', colors: [0x000D00FF, 0x003300FF, 0x00AA00FF, 0x00FF00FF, 0x80FF80FF] },
  
  // RGB
  { name: 'rgb', colors: [0x000000FF, 0xFF0000FF, 0x00FF00FF, 0x0000FFFF, 0xFFFF00FF, 0xFF00FFFF, 0x00FFFFFF, 0xFFFFFFFF] },
  { name: 'cmyk', colors: [0xFFFFFFFF, 0x00FFFFFF, 0xFF00FFFF, 0xFFFF00FF, 0x000000FF] },
  
  // Artistic
  { name: 'ink', colors: [0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x666666FF, 0xCCCCCCFF] },
  { name: 'pastel', colors: [0xFFE0E0FF, 0xFFE0FFFF, 0xE0FFE0FF, 0xE0FFFFE0, 0xFFFFE0FF, 0xFFE0FFFF] },
  { name: 'vintage', colors: [0x2A1A0DFF, 0x4A3A1AFF, 0x8A6A3AFF, 0xCAB88AFF, 0xEDE8D4FF] },
  { name: 'noir', colors: [0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x4A4A4AFF, 0x666666FF] },
  
  // High contrast
  { name: 'high-contrast', colors: [0x000000FF, 0xFFFFFFFF] },
  { name: 'red-scale', colors: [0x000000FF, 0x330000FF, 0x660000FF, 0x990000FF, 0xCC0000FF, 0xFF0000FF] },
  { name: 'green-scale', colors: [0x000000FF, 0x003300FF, 0x006600FF, 0x009900FF, 0x00CC00FF, 0x00FF00FF] },
  { name: 'blue-scale', colors: [0x000000FF, 0x000033FF, 0x000066FF, 0x000099FF, 0x0000CCFF, 0x0000FFFF] },
  
  // Special
  { name: 'rainbow', colors: [0xFF0000FF, 0xFF7F00FF, 0xFFFF00FF, 0x00FF00FF, 0x0000FFFF, 0x4B0082FF, 0x9400D3FF] },
  { name: 'fire', colors: [0x000000FF, 0x330000FF, 0x661100FF, 0x993300FF, 0xCC5500FF, 0xFF7700FF, 0xFF9900FF, 0xFFCC00FF] },
  { name: 'electric', colors: [0x00001AFF, 0x00004DFF, 0x000080FF, 0x0000B3FF, 0x0000E6FF, 0x0033FFFF, 0x0066FFFF, 0x0099FFFF] },
  { name: 'royal', colors: [0x1A0033FF, 0x330066FF, 0x4D0099FF, 0x6600CCFF, 0x8000FFFF, 0x9933FFFF, 0xB266FFFF, 0xCC99FFFF] },
  { name: 'forest', colors: [0x001A00FF, 0x003300FF, 0x004D00FF, 0x006600FF, 0x008000FF, 0x009900FF, 0x00B300FF, 0x00CC00FF] },
  { name: 'sunset', colors: [0x330000FF, 0x661100FF, 0x993300FF, 0xCC5500FF, 0xFF7700FF, 0xFF9933FF, 0xFFBB66FF, 0xFFDD99FF] },
  { name: 'ocean-deep', colors: [0x000022FF, 0x000044FF, 0x001166FF, 0x002288FF, 0x0033AAFF, 0x0044CCFF, 0x0055EEFF, 0x0066FFFF] },
];

export function applyPalettePreset(presetName: string) {
  const preset = PALETTE_PRESETS.find(p => p.name === presetName);
  if (preset) {
    // Preserve exact color count - don't pad to 6
    setParams('palette', [...preset.colors]);
    setParams('paletteSize', preset.colors.length);
    setParams('colorMode', 2);
  }
}

export const gridCols = createMemo(() => {
  // Will be computed based on canvas size in the component
  return 100; // placeholder
});