import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@solidjs/testing-library';
import { useImage } from '../hooks/useImage';
import { useExport } from '../hooks/useExport';

describe('useImage', () => {
  it('stores image data', () => {
    const { result } = renderHook(() => useImage());
    
    const testData = new Uint8Array([1, 2, 3, 4]);
    
    result.loadImage(testData, 'test.png');
    
    expect(result.imageData()).toEqual(testData);
    expect(result.imageName()).toBe('test.png');
  });

  it('clears image data', () => {
    const { result } = renderHook(() => useImage());
    
    result.loadImage(new Uint8Array([1, 2, 3]), 'test.png');
    
    result.clearImage();
    
    expect(result.imageData()).toBeNull();
    expect(result.imageName()).toBe('');
  });
});

describe('useExport', () => {
  const mockConverter = {
    export: vi.fn().mockResolvedValue({
      png: new Uint8Array([1, 2, 3]),
      webp: new Uint8Array([4, 5, 6]),
      text: 'test',
      html: '<html></html>',
      ansi: 'test',
    }),
  };

  const mockGrid = {
    data: new Uint16Array(),
    cols: 10,
    rows: 10,
    glyph_width: 8,
    glyph_height: 8,
    width_px: 80,
    height_px: 80,
    rgba: new Uint8Array(80 * 80 * 4),
  };

  const mockConverterSignal = vi.fn(() => mockConverter);
  const mockGridSignal = vi.fn(() => mockGrid);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:mock');
    vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
    vi.spyOn(document.body, 'appendChild').mockImplementation(() => {});
    vi.spyOn(document.body, 'removeChild').mockImplementation(() => {});
    const clickMock = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(clickMock);
  });

  it('calls converter export and downloads files', async () => {
    const { result } = renderHook(() => useExport(mockConverterSignal, mockGridSignal));
    
    const params = {
      webpQuality: 85,
      webpLossless: false,
      htmlFg: '#fff',
      htmlBg: '#000',
      htmlFont: 'monospace',
      htmlLineHeight: 1.1,
      htmlFontSize: 12,
      ansiFg: 0xFFFFFFFF,
      ansiBg: 0x000000FF,
    };
    
    await result.exportAll(params);
    
    expect(mockConverter.export).toHaveBeenCalledWith(expect.anything(), params);
    expect(URL.createObjectURL).toHaveBeenCalledTimes(5); // PNG, WebP, Text, HTML, ANSI
  });

  it('sets downloading state', async () => {
    const { result } = renderHook(() => useExport(mockConverterSignal, mockGridSignal));
    
    const params = {
      webpQuality: 85,
      webpLossless: false,
      htmlFg: '#fff',
      htmlBg: '#000',
      htmlFont: 'monospace',
      htmlLineHeight: 1.1,
      htmlFontSize: 12,
      ansiFg: 0xFFFFFFFF,
      ansiBg: 0x000000FF,
    };
    
    expect(result.downloading()).toBe(false);
    
    const promise = result.exportAll(params);
    
    // During export, downloading should be true
    // Note: We can't easily test intermediate state without more complex mocking
    await promise;
    
    expect(result.downloading()).toBe(false);
  });
});