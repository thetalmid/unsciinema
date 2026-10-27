import { createSignal, onMount, createEffect } from 'solid-js';
import wasmInit, { UnsciiConverter, FontVariant } from 'unscii-converter-core';

export function useUnscii() {
  const [converter, setConverter] = createSignal<UnsciiConverter | null>(null);
  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [currentVariant, setCurrentVariant] = createSignal(FontVariant.Unscii8);

  const init = async (variant: FontVariant = FontVariant.Unscii8) => {
    try {
      console.log('init called, variant:', variant);
      await wasmInit();
      console.log('wasmInit done');
      const fontUrl = `/fonts/unscii-${getVariantSuffix(variant)}.hex`;
      console.log('Loading font:', fontUrl);
      const fontResp = await fetch(fontUrl);
      console.log('fontResp.ok:', fontResp.ok);
      if (!fontResp.ok) throw new Error('Failed to load font: ' + fontUrl);
      const fontHex = await fontResp.text();
      console.log('Font loaded, length:', fontHex.length);
      
      const conv = new UnsciiConverter(variant);
      console.log('UnsciiConverter created');
      setConverter(conv);
      setCurrentVariant(variant);
      setReady(true);
      console.log('ready set to true');
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to initialize');
      console.error('init error:', e);
    }
  };

  const setFontVariant = async (variant: FontVariant) => {
    console.log('setFontVariant called:', variant);
    if (variant === currentVariant()) return;
    setReady(false);
    await init(variant);
    console.log('setFontVariant done, ready:', true);
  };

  const getFontAtlas = () => {
    const conv = converter();
    if (conv && conv.font_atlas) {
      const atlas = conv.font_atlas();
      console.log('getFontAtlas returned:', atlas ? atlas.length : 'null');
      return atlas;
    }
    console.log('getFontAtlas: no converter or font_atlas method');
    return null;
  };

  const getFontAtlasSize = () => {
    const conv = converter();
    if (conv && conv.font_atlas_size) {
      const size = conv.font_atlas_size();
      console.log('getFontAtlasSize returned:', size);
      return size;
    }
    return null;
  };

  const getFontVariant = () => {
    const conv = converter();
    if (conv && conv.font_variant) {
      const variant = conv.font_variant();
      console.log('getFontVariant returned:', variant);
      return variant;
    }
    return null;
  };

  return { converter, ready, error, init, setFontVariant, currentVariant, getFontAtlas, getFontAtlasSize, getFontVariant };
}

function getVariantSuffix(variant: FontVariant): string {
  switch (variant) {
    case FontVariant.Unscii8: return '8';
    case FontVariant.Unscii8Alt: return '8-alt';
    case FontVariant.Unscii8Fantasy: return '8-fantasy';
    case FontVariant.Unscii8Mcr: return '8-mcr';
    case FontVariant.Unscii8Tall: return '8-tall';
    case FontVariant.Unscii8Thin: return '8-thin';
    case FontVariant.Unscii16: return '16';
    default: return '8';
  }
}