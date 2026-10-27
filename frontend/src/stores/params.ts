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
  palette: [0x000000, 0xFFFFFF],
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

export const PALETTE_PRESETS = [
  { name: 'monochrome', colors: [0x000000, 0xFFFFFF] },
  { name: 'terminal', colors: [0x000000, 0x333333, 0x666666, 0x999999, 0xCCCCCC, 0xFFFFFF] },
  { name: 'gameboy', colors: [0x0F380F, 0x306230, 0x8BAC0F, 0x9BBC0F] },
  { name: 'sepia', colors: [0x1A1A1A, 0x4A3A2A, 0x8A6A4A, 0xD4B896, 0xF5E6C8] },
  { name: 'campfire', colors: [0x0D0D0D, 0x3D1A0D, 0x8B3A1A, 0xD47A3A, 0xF5C896] },
  { name: 'gold', colors: [0x0D0D00, 0x3D3D00, 0xB8B800, 0xFFF000, 0xFFFF80] },
  { name: 'ocean', colors: [0x000D1A, 0x003D4D, 0x007A8C, 0x4DB8D4, 0x96E6F5] },
  { name: 'ice', colors: [0x00001A, 0x002A6D, 0x006AB8, 0x4DA6E6, 0x96D4F5] },
  { name: 'synthwave', colors: [0x1A0033, 0x4D0066, 0xFF00FF, 0xFF66FF, 0xFFCCFF] },
  { name: 'cyberpunk', colors: [0x0D001A, 0x3D0066, 0xB800FF, 0xFF00FF, 0xFF80FF] },
  { name: 'ink', colors: [0x000000, 0x1A1A1A, 0x333333, 0x666666, 0xCCCCCC] },
  { name: 'rainbow', colors: [0xFF0000, 0xFF7F00, 0xFFFF00, 0x00FF00, 0x0000FF, 0x4B0082, 0x9400D3] },
  { name: 'fire', colors: [0x000000, 0x330000, 0x661100, 0x993300, 0xCC5500, 0xFF7700, 0xFF9900, 0xFFCC00] },
  { name: 'high-contrast', colors: [0x000000, 0xFFFFFF] },
];

export function applyPalettePreset(presetName: string) {
  const preset = PALETTE_PRESETS.find(p => p.name === presetName);
  if (preset) {
    setParams('palette', [...preset.colors, 0, 0, 0, 0, 0, 0].slice(0, 6));
    setParams('paletteSize', preset.colors.length);
    setParams('colorMode', 2);
  }
}

export const gridCols = createMemo(() => {
  // Will be computed based on canvas size in the component
  return 100; // placeholder
});