import { onMount, onCleanup } from 'solid-js';

export interface CanvasProps {
  ref: (el: HTMLCanvasElement) => void;
  onResize?: () => void;
}

export default function Canvas(props: CanvasProps) {
  let canvasEl: HTMLCanvasElement | null = null;

  onMount(() => {
    const updateSize = () => {
      if (canvasEl) {
        const rect = canvasEl.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const newWidth = Math.round(rect.width * dpr);
        const newHeight = Math.round(rect.height * dpr);
        
        if (canvasEl.width !== newWidth || canvasEl.height !== newHeight) {
          canvasEl.width = newWidth;
          canvasEl.height = newHeight;
        }
        
        // Trigger redraw after size update
        if (props.onResize) {
          props.onResize();
        }
      }
    };
    
    updateSize();
    window.addEventListener('resize', updateSize);
    onCleanup(() => window.removeEventListener('resize', updateSize));
  });

  return (
    <canvas
      ref={(el) => {
        canvasEl = el;
        props.ref(el);
      }}
      class="canvas"
      style={{ width: '100%', height: '100%' }}
    />
  );
}