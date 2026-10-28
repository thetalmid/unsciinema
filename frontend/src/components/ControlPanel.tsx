import { createSignal } from 'solid-js';
import { params, setParams, DITHER_ALGORITHMS, FONT_VARIANTS, COLOR_MODES, PALETTE_PRESETS, applyPalettePreset } from '../stores/params';

export default function ControlPanel(props: {
  params: any;
  setParams: any;
  onConvert: () => void;
  onExport: () => void;
  disabled: boolean;
  converting: boolean;
}) {
  const [activeTab, setActiveTab] = createSignal<'dither' | 'font' | 'render' | 'color' | 'advanced'>('dither');

  return (
    <div class="control-panel">
      <div class="tabs">
        {['dither', 'font', 'render', 'color', 'advanced'].map(tab => (
          <button
            class={tab === activeTab() ? 'active' : ''}
            onClick={() => setActiveTab(tab as any)}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div class="tab-content">
        {activeTab() === 'dither' && (
          <div class="section">
            <label>
              <span>Algorithm</span>
              <select
                value={props.params.ditherAlgorithm}
                onChange={e => props.setParams('ditherAlgorithm', parseInt(e.currentTarget.value))}
                disabled={props.disabled}
              >
                {DITHER_ALGORITHMS.map(a => <option value={a.value}>{a.label}</option>)}
              </select>
            </label>

            <label>
              <span>Strength: {props.params.ditherStrength.toFixed(2)}</span>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={props.params.ditherStrength}
                onInput={e => props.setParams('ditherStrength', parseFloat(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>

            <label>
              <span>Threshold: {props.params.threshold.toFixed(2)}</span>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={props.params.threshold}
                onInput={e => props.setParams('threshold', parseFloat(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>
          </div>
        )}

        {activeTab() === 'font' && (
          <div class="section">
            <label>
              <span>Font Variant</span>
              <select
                value={props.params.fontVariant}
                onChange={e => props.setParams('fontVariant', parseInt(e.currentTarget.value))}
                disabled={props.disabled}
              >
                {FONT_VARIANTS.map(f => <option value={f.value}>{f.label}</option>)}
              </select>
            </label>

            <label>
              <span>Cell Size: {props.params.cellSize.toFixed(1)}px</span>
              <input
                type="range"
                min="4"
                max="32"
                step="0.5"
                value={props.params.cellSize}
                onInput={e => props.setParams('cellSize', parseFloat(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>
          </div>
        )}

        {activeTab() === 'render' && (
          <div class="section">
            <label>
              <span>Color Mode</span>
              <select
                value={props.params.colorMode}
                onChange={e => props.setParams('colorMode', parseInt(e.currentTarget.value))}
                disabled={props.disabled}
              >
                {COLOR_MODES.map(c => <option value={c.value}>{c.label}</option>)}
              </select>
            </label>

            <label>
              <span>Contrast: {props.params.contrast.toFixed(2)}</span>
              <input
                type="range"
                min="0.1"
                max="3"
                step="0.05"
                value={props.params.contrast}
                onInput={e => props.setParams('contrast', parseFloat(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>

            <label>
              <span>Gamma: {props.params.gamma.toFixed(2)}</span>
              <input
                type="range"
                min="0.5"
                max="2.5"
                step="0.05"
                value={props.params.gamma}
                onInput={e => props.setParams('gamma', parseFloat(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>
          </div>
        )}

        {activeTab() === 'color' && (
          <div class="section">
            <h4>Preset Palettes</h4>
            <div class="palette-grid">
              {PALETTE_PRESETS.map(preset => (
                <button
                  key={preset.name}
                  class="palette-btn"
                  onClick={() => applyPalettePreset(preset.name)}
                  disabled={props.disabled}
                  title={preset.name}
                >
                  <div class="palette-preview">
                    {preset.colors.slice(0, 6).map((color, i) => (
                      <div
                        key={i}
                        class="color-swatch"
                        style={{ backgroundColor: `#${color.toString(16).padStart(8, '0').slice(0, 6)}` }}
                      />
                    ))}
                  </div>
                  <span>{preset.name}</span>
                </button>
              ))}
            </div>

            {props.params.colorMode === 2 && (
              <div class="custom-palette">
                <h4>Custom Palette (up to 6 colors)</h4>
                <div class="color-inputs">
                  {Array.from({ length: 6 }, (_, i) => (
                    <input
                      key={i}
                      type="color"
                      value={`#${(props.params.palette[i] >>> 8).toString(16).padStart(6, '0')}`}
                      onChange={e => {
                        const hex = e.currentTarget.value.slice(1);
                        const rgba = parseInt(hex + 'FF', 16);
                        const newPalette = [...props.params.palette];
                        newPalette[i] = rgba;
                        props.setParams('palette', newPalette);
                      }}
                      disabled={props.disabled}
                    />
                  ))}
                </div>
                <label>
                  <span>Palette Size: {props.params.paletteSize}</span>
                  <input
                    type="range"
                    min="2"
                    max="6"
                    step="1"
                    value={props.params.paletteSize}
                    onInput={e => props.setParams('paletteSize', parseInt(e.currentTarget.value))}
                    disabled={props.disabled}
                  />
                </label>
              </div>
            )}
          </div>
        )}

        {activeTab() === 'advanced' && (
          <div class="section">
            <label>
              <span>Target Width: {props.params.targetWidth || 'Auto'}px</span>
              <input
                type="range"
                min="0"
                max="2000"
                step="50"
                value={props.params.targetWidth}
                onInput={e => props.setParams('targetWidth', parseInt(e.currentTarget.value))}
                disabled={props.disabled}
              />
            </label>
          </div>
        )}

      </div>

      <div class="actions">
        <button 
          class="primary" 
          onClick={props.onConvert} 
          disabled={props.disabled || props.converting}
        >
          {props.converting ? 'Converting...' : 'Convert'}
        </button>
        <button 
          class="secondary" 
          onClick={props.onExport} 
          disabled={props.disabled}
        >
          Export
        </button>
      </div>
    </div>
  );
}