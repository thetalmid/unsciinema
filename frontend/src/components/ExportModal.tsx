import { createSignal, createEffect } from 'solid-js';
import { ExportParams } from '../hooks/useExport';

export default function ExportModal(props: {
  show: boolean | (() => boolean);
  onClose: () => void;
  onConfirm: (params: ExportParams) => void;
  disabled: boolean | (() => boolean);
}) {
  const isShow = typeof props.show === 'function' ? props.show : () => props.show;
  const isDisabled = typeof props.disabled === 'function' ? props.disabled : () => props.disabled;

  const [exportParams, setExportParams] = createSignal<ExportParams>({
    webpQuality: 85,
    webpLossless: false,
    htmlFg: '#ffffff',
    htmlBg: '#000000',
    htmlFont: "'Unscii', monospace",
    htmlLineHeight: 1.1,
    htmlFontSize: 12,
    ansiFg: 0xFFFFFFFF,
    ansiBg: 0x000000FF,
  });

  createEffect(() => {
    if (props.show) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
  });

  if (!isShow()) return null;

  return (
    <div class="modal-overlay" onClick={props.onClose}>
      <div class="modal" onClick={e => e.stopPropagation()}>
        <div class="modal-header">
          <h3>Export Options</h3>
          <button class="close-btn" onClick={props.onClose}>✕</button>
        </div>
        
        <div class="modal-content">
          <div class="section">
            <h4>WebP</h4>
            <label>
              <span>Quality: {exportParams().webpQuality}</span>
              <input
                type="range"
                min="1"
                max="100"
                value={exportParams().webpQuality}
                onInput={e => setExportParams(p => ({ ...p, webpQuality: parseInt(e.currentTarget.value) }))}
              />
            </label>
            <label class="checkbox">
              <input
                type="checkbox"
                checked={exportParams().webpLossless}
                onChange={e => setExportParams(p => ({ ...p, webpLossless: e.currentTarget.checked }))}
              />
              <span>Lossless</span>
            </label>
          </div>

          <div class="section">
            <h4>HTML</h4>
            <label>
              <span>Foreground</span>
              <input
                type="color"
                value={exportParams().htmlFg}
                onChange={e => setExportParams(p => ({ ...p, htmlFg: e.currentTarget.value }))}
              />
            </label>
            <label>
              <span>Background</span>
              <input
                type="color"
                value={exportParams().htmlBg}
                onChange={e => setExportParams(p => ({ ...p, htmlBg: e.currentTarget.value }))}
              />
            </label>
            <label>
              <span>Font: {exportParams().htmlFont}</span>
              <select
                value={exportParams().htmlFont}
                onChange={e => setExportParams(p => ({ ...p, htmlFont: e.currentTarget.value }))}
              >
                <option value="'Unscii', monospace">Unscii (web font)</option>
                <option value="monospace">Monospace</option>
                <option value="'Courier New', monospace">Courier New</option>
                <option value="'Fira Code', monospace">Fira Code</option>
              </select>
            </label>
            <label>
              <span>Line Height: {exportParams().htmlLineHeight.toFixed(1)}</span>
              <input
                type="range"
                min="0.8"
                max="2"
                step="0.1"
                value={exportParams().htmlLineHeight}
                onInput={e => setExportParams(p => ({ ...p, htmlLineHeight: parseFloat(e.currentTarget.value) }))}
              />
            </label>
            <label>
              <span>Font Size: {exportParams().htmlFontSize}px</span>
              <input
                type="range"
                min="8"
                max="24"
                step="1"
                value={exportParams().htmlFontSize}
                onInput={e => setExportParams(p => ({ ...p, htmlFontSize: parseInt(e.currentTarget.value) }))}
              />
            </label>
          </div>

          <div class="section">
            <h4>ANSI</h4>
            <label>
              <span>Foreground</span>
              <input
                type="color"
                value={`#${(exportParams().ansiFg >> 8).toString(16).padStart(6, '0')}`}
                onChange={e => setExportParams(p => ({ ...p, ansiFg: parseInt(e.currentTarget.value.slice(1) + 'FF', 16) }))}
              />
            </label>
            <label>
              <span>Background</span>
              <input
                type="color"
                value={`#${(exportParams().ansiBg >> 8).toString(16).padStart(6, '0')}`}
                onChange={e => setExportParams(p => ({ ...p, ansiBg: parseInt(e.currentTarget.value.slice(1) + 'FF', 16) }))}
              />
            </label>
          </div>
        </div>

        <div class="modal-actions">
          <button class="secondary" onClick={props.onClose}>Cancel</button>
          <button class="primary" onClick={() => props.onConfirm(exportParams())} disabled={isDisabled()}>
            Export All Formats
          </button>
        </div>
      </div>
    </div>
  );
}