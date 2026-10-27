import { createSignal } from 'solid-js';

export default function FileDrop(props: {
  onLoad: (file: File) => void;
  disabled: boolean;
  imageName: string;
  onClear: () => void;
}) {
  const [dragOver, setDragOver] = createSignal(false);

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };

  const handleDrop = (e: DragEvent) => {
    console.log('handleDrop event:', e);
    e.preventDefault();
    setDragOver(false);
    const file = e.dataTransfer?.files[0];
    console.log('dropped file:', file);
    if (file && file.type.startsWith('image/')) {
      console.log('calling onLoad with:', file);
      props.onLoad(file);
    }
  };

  const handleFileSelect = (e: Event) => {
    console.log('handleFileSelect event:', e);
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    console.log('selected file:', file);
    if (file) {
      console.log('calling onLoad with:', file);
      props.onLoad(file);
      input.value = '';
    }
  };

  if (props.imageName) {
    return (
      <div class="file-drop loaded">
        <span class="filename">{props.imageName}</span>
        <button class="clear-btn" onClick={props.onClear} disabled={props.disabled}>✕</button>
      </div>
    );
  }

  return (
    <div 
      class={`file-drop ${dragOver() ? 'drag-over' : ''}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <input
        type="file"
        accept="image/*"
        onChange={handleFileSelect}
        class="file-input"
        disabled={props.disabled}
        id="file-input"
      />
      <label for="file-input" class="drop-label">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
        <p>Drop an image here or click to browse</p>
        <span class="hint">PNG, JPEG, GIF, WebP</span>
      </label>
    </div>
  );
}