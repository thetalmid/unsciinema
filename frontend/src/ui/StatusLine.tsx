/** Global status line (aria-live) for loading / convert / export / errors. */
import { Show } from "solid-js";
import {
  gifProgress,
  lastError,
  phase,
  resultSeq,
  statusMessage,
} from "../state/app";

export function StatusLine() {
  return (
    <div
      class="status-line"
      role="status"
      aria-live="polite"
      data-testid="status"
      data-result-seq={resultSeq()}
    >
      <span class={`phase phase-${phase()}`}>{phase()}</span>
      <span class="status-msg">{statusMessage()}</span>
      <Show when={gifProgress()}>
        {(p) => (
          <span class="status-sub">
            GIF {p().done}/{p().total}
          </span>
        )}
      </Show>
      <Show when={lastError()}>
        {(err) => (
          <span class="status-err" role="alert">
            {err()}
          </span>
        )}
      </Show>
    </div>
  );
}
