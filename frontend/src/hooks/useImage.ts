import { createSignal } from 'solid-js';

export function useImage() {
  const [imageData, setImageData] = createSignal<Uint8Array | null>(null);
  const [imageName, setImageName] = createSignal<string>('');

  const loadImage = (data: Uint8Array, name?: string) => {
    setImageData(data);
    setImageName(name || 'image');
  };

  const clearImage = () => {
    setImageData(null);
    setImageName('');
  };

  return { imageData, imageName, loadImage, clearImage };
}