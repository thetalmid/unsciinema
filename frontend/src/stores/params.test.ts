import { describe, it, expect, vi, beforeEach } from 'vitest';
import { params, setParams, DITHER_ALGORITHMS, FONT_VARIANTS, COLOR_MODES, PALETTE_PRESETS, applyPalettePreset } from '../stores/params';

describe('params store', () => {
  beforeEach(() => {
    // Reset to defaults
    setParams('ditherAlgorithm', 0);
    setParams('ditherStrength', 1.0);
    setParams('threshold', 0.5);
    setParams('fontVariant', 0);
    setParams('cellSize', 8.0);
    setParams('colorMode', 0);
    setParams('palette', [0x000000FF, 0xFFFFFFFF, 0, 0, 0, 0]);
    setParams('paletteSize', 2);
    setParams('alphaMode', 0);
    setParams('contrast', 1.0);
    setParams('gamma', 1.0);
    setParams('targetWidth', 0);
    setParams('crtScanlines', false);
    setParams('crtCurvature', 0.0);
    setParams('crtBloom', 0.0);
    setParams('crtVignette', 0.0);
  });

  it('has correct default values', () => {
    expect(params.ditherAlgorithm).toBe(0);
    expect(params.ditherStrength).toBe(1.0);
    expect(params.threshold).toBe(0.5);
    expect(params.fontVariant).toBe(0);
    expect(params.cellSize).toBe(8.0);
    expect(params.colorMode).toBe(0);
    expect(params.paletteSize).toBe(2);
    expect(params.contrast).toBe(1.0);
    expect(params.gamma).toBe(1.0);
  });

  it('updates individual params', () => {
    setParams('ditherAlgorithm', 1);
    expect(params.ditherAlgorithm).toBe(1);

    setParams('cellSize', 16.0);
    expect(params.cellSize).toBe(16.0);

    setParams('colorMode', 2);
    expect(params.colorMode).toBe(2);
  });

  it('updates palette correctly', () => {
    const newPalette = [0xFF0000FF, 0x00FF00FF, 0x0000FFFF, 0xFFFF00FF, 0, 0];
    setParams('palette', newPalette);
    setParams('paletteSize', 4);
    
    expect(params.palette).toEqual(newPalette);
    expect(params.paletteSize).toBe(4);
  });
});

describe('constants', () => {
  it('DITHER_ALGORITHMS has 10 entries', () => {
    expect(DITHER_ALGORITHMS.length).toBe(10);
    expect(DITHER_ALGORITHMS[0].value).toBe(0);
    expect(DITHER_ALGORITHMS[9].value).toBe(9);
  });

  it('FONT_VARIANTS has 7 entries', () => {
    expect(FONT_VARIANTS.length).toBe(7);
  });

  it('COLOR_MODES has 3 entries', () => {
    expect(COLOR_MODES.length).toBe(3);
  });

  it('PALETTE_PRESETS has 31 entries', () => {
    expect(PALETTE_PRESETS.length).toBe(31);
  });
});

describe('applyPalettePreset', () => {
  beforeEach(() => {
    setParams('colorMode', 0);
    setParams('palette', [0, 0, 0, 0, 0, 0]);
    setParams('paletteSize', 0);
  });

  it('applies gameboy preset', () => {
    applyPalettePreset('gameboy');
    expect(params.colorMode).toBe(2);
    expect(params.paletteSize).toBe(4);
    expect(params.palette[0]).toBe(0x0F380FFF);
    expect(params.palette[3]).toBe(0x9BBC0FFF);
  });

  it('applies synthwave preset', () => {
    applyPalettePreset('synthwave');
    expect(params.colorMode).toBe(2);
    expect(params.paletteSize).toBe(5);
    expect(params.palette[0]).toBe(0x1A0033FF);
    expect(params.palette[4]).toBe(0xFFCCFFFF);
  });

  it('applies monochrome preset', () => {
    applyPalettePreset('monochrome');
    expect(params.paletteSize).toBe(2);
    expect(params.palette[0]).toBe(0x000000FF);
    expect(params.palette[1]).toBe(0xFFFFFFFF);
  });

  it('does nothing for unknown preset', () => {
    applyPalettePreset('nonexistent');
    expect(params.colorMode).toBe(0);
    expect(params.paletteSize).toBe(0);
  });
});