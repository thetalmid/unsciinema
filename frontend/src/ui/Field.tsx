/** Small accessible form primitives shared by the control panel. */
import type { JSX } from "@solidjs/web";

export function Group(props: { title: string; children: JSX.Element; note?: string }) {
  return (
    <section class="group" aria-label={props.title}>
      <h2 class="group-title">{props.title}</h2>
      <div class="group-body">{props.children}</div>
      {props.note ? <p class="group-note">{props.note}</p> : null}
    </section>
  );
}

export function Slider(props: {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onInput: (v: number) => void;
  display?: (v: number) => string;
  disabled?: boolean;
  hint?: string;
}) {
  return (
    <div class="field">
      <div class="field-head">
        <label for={props.id}>{props.label}</label>
        <output for={props.id} class="field-value">
          {props.display ? props.display(props.value) : props.value.toFixed(2)}
        </output>
      </div>
      <input
        id={props.id}
        type="range"
        min={props.min}
        max={props.max}
        step={props.step}
        value={props.value}
        disabled={props.disabled ?? false}
        aria-describedby={props.hint ? `${props.id}-hint` : undefined}
        onInput={(e) => props.onInput(Number(e.currentTarget.value))}
      />
      {props.hint ? (
        <p id={`${props.id}-hint`} class="field-hint">
          {props.hint}
        </p>
      ) : null}
    </div>
  );
}

export function Select(props: {
  id: string;
  label: string;
  value: number;
  options: ReadonlyArray<{ value: number; label: string }>;
  onChange: (v: number) => void;
  disabled?: boolean;
  hint?: string;
}) {
  return (
    <div class="field">
      <label for={props.id}>{props.label}</label>
      <select
        id={props.id}
        value={props.value}
        disabled={props.disabled ?? false}
        aria-describedby={props.hint ? `${props.id}-hint` : undefined}
        onChange={(e) => props.onChange(Number(e.currentTarget.value))}
      >
        {props.options.map((o) => (
          <option value={o.value}>{o.label}</option>
        ))}
      </select>
      {props.hint ? (
        <p id={`${props.id}-hint`} class="field-hint">
          {props.hint}
        </p>
      ) : null}
    </div>
  );
}

export function NumberField(props: {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onInput: (v: number) => void;
  hint?: string;
}) {
  return (
    <div class="field">
      <label for={props.id}>{props.label}</label>
      <input
        id={props.id}
        type="number"
        min={props.min}
        max={props.max}
        step={props.step ?? 1}
        value={props.value}
        aria-describedby={props.hint ? `${props.id}-hint` : undefined}
        onInput={(e) => {
          const v = Number(e.currentTarget.value);
          if (Number.isFinite(v)) props.onInput(v);
        }}
      />
      {props.hint ? (
        <p id={`${props.id}-hint`} class="field-hint">
          {props.hint}
        </p>
      ) : null}
    </div>
  );
}
