import { createSignal, onMount, onCleanup } from 'solid-js';

export default function Canvas(props: { ref: (el: HTMLCanvasElement) => void }) {
  const [size, setSize] = createSignal({ width: 800, height: 600 });

  onMount(() => {
    const updateSize = () => {
      const canvas = document.querySelector('canvas');
      if (canvas) {
        const rect = canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        setSize({ width: rect.width * dpr, height: rect.height * dpr });
      }
    };
    updateSize();
    window.addEventListener('resize', updateSize);
    onCleanup(() => window.removeEventListener('resize', updateSize));
  });

  return (
    <canvas
      ref={props.ref}
      class="canvas"
      width={size().width}
      height={size().height}
      style={{ width: '100%', height: '100%' }}
    />
  );
}