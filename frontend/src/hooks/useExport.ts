import { createSignal } from 'solid-js';
import { UnsciiConverter, ExportParams } from 'unscii-converter-core';

export function useExport(converter: () => UnsciiConverter | null) {
  const [downloading, setDownloading] = createSignal(false);

  const downloadBlob = (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const exportAll = async (params: ExportParams) => {
    const conv = converter();
    if (!conv) throw new Error('Converter not ready');
    
    setDownloading(true);
    try {
      const result = conv.export(null!, params);
      
      // Download all formats
      downloadBlob(new Blob([result.png], { type: 'image/png' }), 'unscii.png');
      downloadBlob(new Blob([result.webp], { type: 'image/webp' }), 'unscii.webp');
      downloadBlob(new Blob([result.text], { type: 'text/plain' }), 'unscii.txt');
      downloadBlob(new Blob([result.html], { type: 'text/html' }), 'unscii.html');
      downloadBlob(new Blob([result.ansi], { type: 'text/plain' }), 'unscii.ansi');
    } finally {
      setDownloading(false);
    }
  };

  return { exportAll, downloading };
}