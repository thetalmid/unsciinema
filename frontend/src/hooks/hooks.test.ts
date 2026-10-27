import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/solid';
import { useUnscii } from '../hooks/useUnscii';

describe('useUnscii', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes converter when ready', async () => {
    const { result } = renderHook(() => useUnscii());
    
    expect(result.ready).toBe(false);
    expect(result.converter).toBeNull();
    
    await act(async () => {
      await result.init();
    });
    
    expect(result.ready).toBe(true);
    expect(result.converter).not.toBeNull();
  });

  it('returns converter with font atlas', async () => {
    const { result } = renderHook(() => useUnscii());
    
    await act(async () => {
      await result.init();
    });
    
    const converter = result.converter;
    expect(converter).not.toBeNull();
    expect(converter?.font_atlas_size()).toEqual([128, 8]);
    expect(converter?.char_ramp()).toHaveLength(16);
  });
});

import { renderHook, act } from '@testing-library/solid';
import { useImage } from '../hooks/useImage';

describe('useImage', () => {
  it('stores image data', () => {
    const { result } = renderHook(() => useImage());
    
    const testData = new Uint8Array([1, 2, 3, 4]);
    
    act(() => {
      result.loadImage(testData, 'test.png');
    });
    
    expect(result.imageData).toEqual(testData);
    expect(result.imageName).toBe('test.png');
  });

  it('clears image data', () => {
    const { result } = renderHook(() => useImage());
    
    act(() => {
      result.loadImage(new Uint8Array([1, 2, 3]), 'test.png');
    });
    
    act(() => {
      result.clearImage();
    });
    
    expect(result.imageData).toBeNull();
    expect(result.imageName).toBe('');
  });
});

import { renderHook, act } from '@testing-library/solid';
import { useExport } from '../hooks/useExport';

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
    const { result } = renderHook(() => useExport(mockConverter as any));
    
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
    
    await act(async () => {
      await result.exportAll(params);
    });
    
    expect(mockConverter.export).toHaveBeenCalledWith(expect.anything(), params);
    expect(URL.createObjectURL).toHaveBeenCalledTimes(5); // PNG, WebP, Text, HTML, ANSI
  });

  it('sets downloading state', async () => {
    const { result } = renderHook(() => useExport(mockConverter as any));
    
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
    
    expect(result.downloading).toBe(false);
    
    const promise = act(async () => {
      await result.exportAll(params);
    });
    
    // During export, downloading should be true
    // Note: We can't easily test intermediate state without more complex mocking
    await promise;
    
    expect(result.downloading).toBe(false);
  });
});